use rand::thread_rng;
use rand::seq::SliceRandom;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::thread::JoinHandle;
use std::env;
use std::str::FromStr;
use indicatif::ProgressBar;

fn check_sorted(items: &Vec<isize>) -> bool {
    let mut result = true;
    for (i, item) in items[1..].iter().enumerate() {
        if item < &items[i] {
            result = false;
        }
    }
    result
}

fn bogo_sort(list: Vec<isize>, threads: usize) -> (Vec<isize>, Vec<JoinHandle<()>>) {

    let result = Arc::new(Mutex::new(None));

    let mut handles = vec![];
    for _ in 0..threads {
        let list_clone = list.clone();
        let result_clone = Arc::clone(&result);
        let handle = thread::spawn(move || {

            let mut rng = thread_rng();
            let mut temp_items = list_clone.clone();

            while !check_sorted(&temp_items) {
                temp_items.shuffle(&mut rng);
            }
            let mut final_result = result_clone.lock().unwrap();
            *final_result = Some(temp_items);

        });
        handles.push(handle);
    }

    let output: Vec<isize>;
    'main: loop {
        let received = &*result.lock().unwrap();
        match received {
            Some(sorted) => {
                output = sorted.to_vec();
                break 'main;
            },
            None => {}
        }
    }
    (output, handles)
}

fn safe_bogo_sort(list: Vec<isize>, threads: usize) -> Vec<isize> {

    let result = Arc::new(Mutex::new(None));
    let event = Arc::new(Mutex::new(false));

    let mut handles = vec![];
    for _ in 0..threads {
        let list_clone = list.clone();

        let result_clone = Arc::clone(&result);
        let event_clone = Arc::clone(&event);

        let handle = thread::spawn(move || {

            let mut rng = thread_rng();
            let mut temp_items = list_clone.clone();

            'thread_loop: loop {
                let check = check_sorted(&temp_items);
                let events = *(event_clone.lock().unwrap());
                if check {
                    let mut final_result = result_clone.lock().unwrap();
                    *final_result = Some(temp_items);
                    break 'thread_loop;
                }
                match events {
                    true => {break 'thread_loop;},
                    false => temp_items.shuffle(&mut rng)
                }
            }

        });
        handles.push(handle);
    }

    let output: Vec<isize>;
    'main: loop {
        let received = &*result.lock().unwrap();
        match received {
            Some(sorted) => {
                output = sorted.to_vec();
                let mut modifier = event.lock().unwrap();
                *modifier = true;
                break 'main
            },
            None => {}
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }

    output
}

fn main() {

    let args: Vec<String> = env::args().collect();

    let list_length = usize::from_str(&args[1]).unwrap();
    let repetitions = usize::from_str(&args[2]).unwrap();
    let thread_count = usize::from_str(&args[3]).unwrap();

    println!("Length of list: {}", list_length);
    println!("Amount of sorts: {}", repetitions);
    println!("Thread used: {}", thread_count);

    let list: Vec<isize> = (0..list_length).rev().map(|x| x as isize).collect();

    // METHOD 1
    println!("\nMETHOD 1 (hanging threads)");

    let mut times = vec![];

    let bar = ProgressBar::new(repetitions.try_into().unwrap());
    bar.tick();
    for _ in 0..repetitions {

        let now = Instant::now();
        let (_, handles) = bogo_sort(list.clone(), thread_count);
        let elapsed_time = now.elapsed();

        times.push(elapsed_time.as_micros());
        for handle in handles {
            handle.join().unwrap();
        }

        bar.inc(1);

    }
    bar.finish();

    let mut sum = 0;
    for time in times.iter() {
        sum += time;
    }
    println!("Total time: {}µs", sum);
    let average = sum as usize / times.len();
    println!("Average time: {}µs", average);

    // METHOD 2
    println!("\nMETHOD 2 (safely closed threads)");

    let mut times = vec![];

    let bar = ProgressBar::new(repetitions.try_into().unwrap());
    for _ in 0..repetitions {

        let now = Instant::now();
        let _ = safe_bogo_sort(list.clone(), thread_count);
        let elapsed_time = now.elapsed();
        times.push(elapsed_time.as_micros());

        bar.inc(1);

    }
    bar.finish();

    let mut sum = 0;
    for time in times.iter() {
        sum += time;
    }
    println!("Total time: {}µs", sum);
    let average = sum as usize / times.len();
    println!("Average time: {}µs", average);

}
