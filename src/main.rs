use buffered_queue_rs::BufferedQueue;
use std::sync::atomic::Ordering;
use std::thread::{self, sleep};
use std::time::Duration;

fn main() {
    let order = Ordering::SeqCst;

    let (producer, consumer) = BufferedQueue::new(3);
    let mut output = Vec::new();

    let producer_handle = thread::spawn(move || {
        println!("initializing producer thread...");

        for num in 1..=10 {
            // mock processing of the input data
            let processed_num = num * num * num;
            sleep(Duration::from_millis(250));

            producer.push(processed_num);
        }
        producer.elements_processed.store(true, order);
    });

    let consumer_handle = thread::spawn(move || {
        println!("initializing consumer thread...");

        loop {
            let Some(num) = consumer.pop() else {
                    println!("exhausted queue, terminating consumer!\n");
                    return;
            };

            // mock consumption of processed data
            sleep(Duration::from_millis(400));

            output.push(num);

            println!(
                "pushed to output num: {}; output_vec len: {}",
                num,
                output.len()
            );
        }
    });

    producer_handle.join().unwrap();
    consumer_handle.join().unwrap();
}
