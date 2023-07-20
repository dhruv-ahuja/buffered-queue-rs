# buffered-queue-rs

## Introduction
This is my attempt at a simple and very naive buffered/synced queue implementation in Rust. The base thread-safe queue implementation takes inspiration from and builds upon [RustBlockingQueue](https://github.com/JimFawcett/RustBlockingQueue).

## Implementation
The BufferedQueue struct uses AtomicBools  to send updates regarding the queue’s state, if there any are any relevant changes, in a thread-safe manner. 
push  and pop  are the core methods that modify the internal queue object, and call the signal_to_threads  method that checks the queue’s state and pushes updates depending on the recent operation performed – a push or a pop.
  
## Use-Case
A buffered queue can be used in a pipeline where there is a producer and a consumer thread, and the producer wants to process the data in chunks. This approach can be useful in cases where the producer pushes data at a rate that is much faster than the consumer’s rate of consumption. In such cases having a pipeline with no limit can lead to high memory usage as the producer will end up pushing a large amount of data in the pipeline without its timely use.
A buffered queue implementation generally will not lead to a greater downtime or increased latencies as the consumer will consume the data at its relatively slow pace regardless of the capacity of the queue. A good balance should be achievable with proper benchmarking.
