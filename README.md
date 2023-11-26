# buffered-queue-rs

## Introduction

This is my attempt at a simple and very naive buffered/synced queue implementation in Rust. The base thread-safe queue implementation takes inspiration from and builds upon [RustBlockingQueue](https://github.com/JimFawcett/RustBlockingQueue).
An in-depth code explanation is available in [my blog post](
https://dhruv-ahuja.github.io/posts/implementing-buffered-queue-in-rust/).
## Implementation

The `BufferedQueue`  struct uses `Condvars`  to efficiently signal about and update the queue’s state, in a thread-safe manner. `push`  and `pop`  methods contain the core implementation logic, they modify the internal double-ended vector data structure, and call the `signal_queue_changes` method that checks the queue’s state and pushes updates depending on the recent operation performed – a push or a pop.

An overview regarding the performance differences between the two synchronization mechanics is available [in this ChatGPT explaination](https://chat.openai.com/share/890e7c2d-37a9-45dc-966b-f42ed4ddad96 "https://chat.openai.com/share/890e7c2d-37a9-45dc-966b-f42ed4ddad96").


## Use-Case

A buffered queue can be used in a pipeline where there is a producer and a consumer thread, and the producer wants to process the data in chunks. This approach can be useful in cases where the producer pushes data at a rate that is much faster than the consumer’s rate of consumption. In such cases having a pipeline with no limit can lead to high memory usage as the producer will end up pushing a large amount of data in the pipeline without its timely use.

A buffered queue implementation generally will not lead to a greater downtime or increased latencies as the consumer will consume the data at its relatively slow pace regardless of the capacity of the queue. A good balance should be achievable with proper benchmarking.
