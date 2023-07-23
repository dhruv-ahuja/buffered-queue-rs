use buffered_queue_rs::BufferedQueue;
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

        for num in 1..=10 {
            // mock processing of the input data
            let processed_num = num * num * num;
            sleep(Duration::from_millis(250));

            queue.push(processed_num);
        }
        queue.elements_processed.store(true, order);
    });

    let consumer_handle = thread::spawn(move || {
        println!("initializing consumer thread...\n");
        let mut output_vec = output.lock().unwrap();

        loop {
            match consumer_queue.pop() {
                None => {
                    println!("exhausted queue, terminating consumer!\n");
                    return;
                }

                Some(num) => {
                    // mock consumption of processed data
                    sleep(Duration::from_millis(400));

                    output_vec.push(num);
                    println!(
                        "pushed to output num: {}; output_vec len: {}",
                        num,
                        output_vec.len()
                    );
                }
            }
        }
    });

    consumer_handle.join().unwrap();
    producer_handle.join().unwrap();
}
