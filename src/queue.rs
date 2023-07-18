use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};

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

    /// signals to the consumer thread that the queue is empty
    is_empty_signal: Condvar,

    /// boolean flag to signify to all threads whether the queue is empty
    is_empty: Mutex<bool>,

    /// signals to producer threads that the queue is at the defined capacity
    is_full_signal: Condvar,

    /// boolean flag to signiy to all threads whether the queue is full
    is_full: Mutex<bool>,
}

impl<T> BufferedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        let is_nonempty_signal = Condvar::new();
        let is_empty = Mutex::new(true);

        let is_full_signal = Condvar::new();
        let is_full = Mutex::new(false);

        Self {
            data: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            is_empty_signal: is_nonempty_signal,
            is_empty,
            is_full_signal,
            is_full,
        }
    }

    /// ensures that the calling function acquires the mutex guard only if the queue has space
    fn ensure_has_space(&self) -> Option<MutexGuard<'_, VecDeque<T>>> {
        let queue = self.data.lock().unwrap();
        if queue.len() == self.capacity {
            return None;
        }
        Some(queue)
    }

    /// pushes an element to the back of the queue, returning `true` to indicate whether the operation was
    /// successful if the queue had space else `false`
    pub fn push(&self, value: T) -> bool {
        match self.ensure_has_space() {
            None => false,
            Some(mut queue) => {
                queue.push_back(value);
                self.signal_full_queue(queue);
                true
            }
        }
    }

    /// checks whether the queue is full and signals to all listening threads, if it wasn't full before
    fn signal_full_queue(&self, queue: MutexGuard<'_, VecDeque<T>>) {
        let is_full = queue.len() == self.capacity;
        let mut existing_state = self.is_full.lock().unwrap();

        if is_full && is_full != *existing_state {
            *existing_state = true;
            self.is_full_signal.notify_all();
        }
    }

    /// pops an element from the queue and returns the output -- `Some(T)` in case of elements being present in the
    /// queue, else `None`
    pub fn pop(&self) -> Option<T> {
        let mut queue = self.data.lock().unwrap();
        let popped_element = queue.pop_front();

        self.signal_empty_queue(queue);
        popped_element
    }

    /// checks whether the queue is empty and signals to the consumer thread, if it wasn't empty before
    fn signal_empty_queue(&self, queue: MutexGuard<'_, VecDeque<T>>) {
        let is_empty = queue.len() == 0;
        let mut existing_state = self.is_empty.lock().unwrap();

        if is_empty && is_empty != *existing_state {
            *existing_state = true;
            self.is_empty_signal.notify_one();
        }
    }
}
