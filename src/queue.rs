use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// Operation represents the performed operation on the queue -- push or pop
enum Operation {
    Push,
    Pop,
}

/// BufferedQueue is a queue implementation with a pre-defined maximum capacity, for workloads where one part of the
///  pipeline is faster than other parts and processes tasks much faster than the other parts' consumption ability.
///
/// A BufferedQueue in such a case can process tasks upto a certain limit and wait for the signal to resume processing,
/// thus reducing chances of data sitting in a pipeline for long durations and reducing the application's memory
/// consumption.
pub struct BufferedQueue<T> {
    /// represents the internal queue implementation, wrapped in a mutex
    pub data: Mutex<VecDeque<T>>,

    /// represents the maximum number of elements allowed in the queue at a given time
    capacity: usize,

    /// signals to the producer thread that the queue is full
    pub is_full_: AtomicBool,

    /// signals to the consumer thread that the queue is empty
    pub is_empty_: AtomicBool,

    /// signals termination for the consumer thread
    pub elements_processed: AtomicBool,
}

impl<T> BufferedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            is_full_: AtomicBool::new(false),
            is_empty_: AtomicBool::new(true),
            elements_processed: AtomicBool::new(false),
        }
    }

    /// ensures that the calling function acquires the mutex guard only if the queue has space
    fn ensure_has_space(&self) -> Option<MutexGuard<'_, VecDeque<T>>> {
        let queue = self.data.lock().unwrap();
        println!("queue has space");
        (queue.len() != self.capacity).then_some(queue)
    }

    /// pushes an element to the back of the queue, returning `true` to indicate whether the operation was
    /// successful if the queue had space else `false`
    pub fn push(&self, value: T) -> bool {
        match self.ensure_has_space() {
            None => false,

            Some(mut queue) => {
                queue.push_back(value);
                self.signal_to_threads(queue, Operation::Push);
                true
            }
        }
    }

    /// pops an element from the queue and returns the output -- `Some(T)` in case of elements being present in the
    /// queue, else `None`
    pub fn pop(&self) -> Option<T> {
        let mut queue = self.data.lock().unwrap();
        let popped_element = queue.pop_front();

        self.signal_to_threads(queue, Operation::Pop);
        popped_element
    }

    fn signal_to_threads(&self, queue: MutexGuard<'_, VecDeque<T>>, operation: Operation) {
        let order = Ordering::Relaxed;

        let is_empty = queue.len() == 0;
        let is_full = queue.len() == self.capacity;

        match operation {
            //  push => (empty -> false, full -> true?)
            Operation::Push => {
                // only update state if queue was previously empty
                if self.is_empty_.load(order) {
                    self.is_empty_.store(false, order);
                };

                if is_full {
                    self.is_full_.store(true, order);
                };
            }

            // pop => (empty -> true?, full -> false)
            Operation::Pop => {
                // only update state if queue was previously full
                if self.is_full_.load(order) {
                    self.is_full_.store(false, order);
                };

                if is_empty {
                    self.is_empty_.store(true, order);
                }
            }
        }
    }
}
