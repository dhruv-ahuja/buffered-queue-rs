# buffered-queue-rs

## Introduction

This is my attempt at a simple and very naive buffered/synced queue implementation in Rust. The base thread-safe queue implementation takes inspiration from and builds upon [RustBlockingQueue]([https://github.com/JimFawcett/RustBlockingQueue](https://github.com/JimFawcett/RustBlockingQueue)).

## Implementation

The `BufferedQueue`  struct uses `Condvars`  to efficiently signal about and update the queue’s state, in a thread-safe manner. This is an upgrade from the old `AtomicBool` approach that relied on arbitrary sleep durations and long polling-esque behaviour.

An overview regarding the performance differences between the two synchronization mechanics  is available [here](https://chat.openai.com/share/890e7c2d-37a9-45dc-966b-f42ed4ddad96 "https://chat.openai.com/share/890e7c2d-37a9-45dc-966b-f42ed4ddad96").

`push`  and `pop`  are the core methods that modify the internal queue object, and call the signal\_to\_threads method that checks the queue’s state and pushes updates depending on the recent operation performed – a push or a pop.

## Use-Case

A buffered queue can be used in a pipeline where there is a producer and a consumer thread, and the producer wants to process the data in chunks. This approach can be useful in cases where the producer pushes data at a rate that is much faster than the consumer’s rate of consumption. In such cases having a pipeline with no limit can lead to high memory usage as the producer will end up pushing a large amount of data in the pipeline without its timely use.

A buffered queue implementation generally will not lead to a greater downtime or increased latencies as the consumer will consume the data at its relatively slow pace regardless of the capacity of the queue. A good balance should be achievable with proper benchmarking.
