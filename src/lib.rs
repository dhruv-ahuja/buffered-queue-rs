use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

/// Operation represents the performed operation on the queue -- push or pop
enum Operation<'a> {
    Push { is_full_flag: MutexGuard<'a, bool> },
    Pop { is_empty_flag: MutexGuard<'a, bool> },
}

/// BufferedQueue is a queue implementation with a pre-defined maximum capacity, for workloads where one part of the
///  pipeline is faster than other parts and processes tasks much faster than the other parts' consumption ability.
///
/// A BufferedQueue in such a case can process tasks upto a certain limit and wait for the signal to resume processing,
/// thus reducing chances of data sitting in a pipeline for long durations and reducing the application's memory
/// consumption.
pub struct BufferedQueue<T> {
    /// represents the internal queue implementation, wrapped in a mutex
    data: Mutex<VecDeque<T>>,

    /// represents the maximum number of elements allowed in the queue at a given time
    capacity: usize,

    /// indicates whether the queue is full
    pub is_full: Mutex<bool>,

    /// signals to producer threads that the queue is full
    pub is_full_signal: Condvar,

    /// indicates whether the queue is empty
    pub is_empty: Mutex<bool>,

    /// signals to consumer threads that the queue is empty
    pub is_empty_signal: Condvar,

    /// signals that the producer queue has processed all its data
    pub elements_processed: AtomicBool,
}

impl<T> BufferedQueue<T> {
    /// returns producer and consumer BufferedQueue implementations
    pub fn new(capacity: usize) -> (Arc<BufferedQueue<T>>, Arc<BufferedQueue<T>>) {
        let data = Self {
            data: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            is_full: Mutex::new(false),
            is_empty: Mutex::new(true),
            is_full_signal: Condvar::new(),
            is_empty_signal: Condvar::new(),
            elements_processed: AtomicBool::new(false),
        };
        let producer = Arc::new(data);
        let consumer = producer.clone();
        (producer, consumer)
    }

    /// pushes an element to the back of the queue, returning `true` to indicate whether the operation was
    /// successful if the queue had space else `false`
    pub fn push(&self, value: T) {
        let mut queue_is_full = self.is_full.lock().unwrap();
        while *queue_is_full {
            queue_is_full = self.is_full_signal.wait(queue_is_full).unwrap();
        }

        let mut queue = self.data.lock().unwrap();
        queue.push_back(value);
        println!("pushed element");
        self.signal_queue_changes(
            queue,
            Operation::Push {
                is_full_flag: queue_is_full,
            },
        );
    }

    // disabling this as we do not need an `is_empty` method in our case
    #[allow(clippy::len_without_is_empty)]

    /// returns the queue's current length
    pub fn len(&self) -> usize {
        let queue = self.data.lock().unwrap();
        queue.len()
    }

    /// pops an element from the queue and returns the output -- `Some(T)` in case of elements being present in the
    /// queue, else `None`
    pub fn pop(&self) -> Option<T> {
        let mut queue_is_empty = self.is_empty.lock().unwrap();
        while *queue_is_empty {
            if self.elements_processed.load(Ordering::SeqCst) {
                return None;
            }
            queue_is_empty = self.is_empty_signal.wait(queue_is_empty).unwrap();
        }

        let mut queue = self.data.lock().unwrap();
        let popped_element = queue.pop_front();
        println!("popped element");

        self.signal_queue_changes(
            queue,
            Operation::Pop {
                is_empty_flag: queue_is_empty,
            },
        );
        popped_element
    }

    /// passes signals regarding the changes to tge queue's state, based on the recent operation type
    fn signal_queue_changes(&self, queue: MutexGuard<'_, VecDeque<T>>, operation: Operation) {
        let is_empty = queue.len() == 0;
        let is_full = queue.len() == self.capacity;

        match operation {
            //  push => (empty -> false, full -> true?)
            Operation::Push { mut is_full_flag } => {
                let mut is_empty_flag = self.is_empty.lock().unwrap();
                if *is_empty_flag {
                    *is_empty_flag = false;
                    println!("set is_empty to false");
                    self.is_empty_signal.notify_all();
                }

                if is_full {
                    *is_full_flag = true;
                    self.is_full_signal.notify_all();
                    println!("set is_full to true");
                }
            }

            // pop => (empty -> true?, full -> false)
            Operation::Pop { mut is_empty_flag } => {
                let mut is_full_flag = self.is_full.lock().unwrap();
                if *is_full_flag {
                    *is_full_flag = false;
                    println!("set is_full to false");
                    self.is_full_signal.notify_all();
                }

                if is_empty {
                    *is_empty_flag = true;
                    self.is_empty_signal.notify_all();
                    println!("set is_empty to true");
                }
            }
        }
    }
}
