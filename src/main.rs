pub mod queue;

use queue::BufferedQueue;
use std::{
    sync::{atomic::Ordering, Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

fn main() {
    let order = Ordering::Relaxed;

    let queue: Arc<BufferedQueue<i32>> = Arc::new(BufferedQueue::new(3));
    let consumer_queue = queue.clone();

    let output = Arc::new(Mutex::new(Vec::new()));

    let producer_handle = thread::spawn(move || {
        println!("initializing producer thread...\n");

        let mut nums = 1..=10;

        loop {
            if queue.is_full_.load(order) {
                sleep(Duration::from_millis(75));
                continue;
            }

            let num;
            if let Some(val) = nums.next() {
                num = val;
            } else {
                println!("\nexhausted range, terminating producer!\n");
                queue.elements_processed.store(true, order);
                return;
            };

            // mock processing of the input data
            let processed_num = num * num * num;
            sleep(Duration::from_millis(250));

            queue.push(processed_num);
            println!("pushed element {}", processed_num);
        }
    });

    let consumer_handle = thread::spawn(move || {
        println!("initializing consumer thread...\n");

        loop {
            if consumer_queue.is_empty_.load(order) {
                if consumer_queue.elements_processed.load(order) {
                    println!("exhausted queue, terminating consumer!\n");
                    return;
                }
                sleep(Duration::from_millis(150));
                continue;
            }

            let mut output_vec = output.lock().unwrap();

            // we can safely unwrap as we know there's an element in the queue
            let num = consumer_queue.pop().unwrap();
            output_vec.push(num);
            println!("pushed to output num: {}", num);
        }
    });

    consumer_handle.join().unwrap();
    producer_handle.join().unwrap();
}
