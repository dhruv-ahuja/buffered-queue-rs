pub mod queue;

use queue::BufferedQueue;
use std::{
    sync::{atomic::Ordering, Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    let order = Ordering::SeqCst;

    let queue: Arc<BufferedQueue<i32>> = Arc::new(BufferedQueue::new(3));
    let consumer_queue = queue.clone();

    let output = Arc::new(Mutex::new(Vec::new()));

    let producer_handle = thread::spawn(move || {
        println!("initializing producer thread...\n");

        let mut nums = 1..=10;

        loop {
            let mut is_full_flag = queue.is_full.lock().unwrap();
            while *is_full_flag {
                println!("\nwaiting for elements to be consumed");
                is_full_flag = queue.is_full_signal.wait(is_full_flag).unwrap();
            }
            // release the lock
            drop(is_full_flag);

            let num;
            if let Some(val) = nums.next() {
                num = val;
            } else {
                println!("exhausted range, terminating producer!\n");
                queue.elements_processed.store(true, order);
                return;
            }

            // mock processing of the input data
            let processed_num = num * num * num;
            sleep(Duration::from_millis(250));

            queue.push(processed_num);
        }
    });

    let consumer_handle = thread::spawn(move || {
        println!("initializing consumer thread...\n");

        loop {
            if consumer_queue.elements_processed.load(order) {
                println!("exhausted queue, terminating consumer!\n");
                return;
            }

            let mut is_empty_flag = consumer_queue.is_empty.lock().unwrap();
            while *is_empty_flag {
                println!("\nwaiting for elements to be pushed");
                is_empty_flag = consumer_queue.is_empty_signal.wait(is_empty_flag).unwrap();
            }
            // release the lock
            drop(is_empty_flag);

            // we can safely unwrap as we know there's an element in the queue
            let num = consumer_queue.pop().unwrap();

            // mock consumption of processed data
            sleep(Duration::from_millis(400));

            let mut output_vec = output.lock().unwrap();
            output_vec.push(num);
            println!("pushed to output num: {}", num);
        }
    });

    consumer_handle.join().unwrap();
    producer_handle.join().unwrap();
}
