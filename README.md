### Playing w rust

### Graph Package
In this package there are 3 different implementations of `ComputableGraph`: A normal (serial) implementation, and 2 concurrent ones.

**Logic Flow**\
At a very high level, the flow is as follow: given a user defined graph (concurrent or not)
- We build a new internal representation so that instead of having the parent node pointing to the children, we have child nodes pointing to theirs respective parents.
 This makes the computation easier. 
 The concurrent graph is just an extension of the normal one except that we use `Arc` instead of `Rc`. 
- Both graph can work on a datum or a batch of data. 

### *Parallelism Design*
**The `ConcurrentComputeGraph`**\
For a single datum or batch of data, each node is ran within the same thread.  
The user if this API can then divide its data so that each datum, can run in a different thread.
The concurrent graph is therefore optimized for **throughput**. 
This approach has the following pros:
1. Have a perfect maximization of the CPU usage with minimal interrupts.\
If we're executing our code on a 4 core machines, we can have 4 batch running at the time. Every time a batch is completed, a Thread is taking the next batch to process from a queue.

2. Performance is very predictable\
For applications with strict SLAs, predictability in performance is key. You want your performance to have very little variance.
If each datum is executed in a single thread, then approximating the performance of a single call to `ConcurrentComputeGraph.apply` should be about the same as approximating whether it is executed in mutli-threaded context or not. 

3. Optimizations\
If the performance is predictable, and the execution flow for a single datum is constently the same, then it becomes easier to optimize and profile an application.
We might want to say that some operations can be vectorized and use SIMD instructions.

All the above pros hold when the operators in a graph are CPU bounded. 
If we were to have operators with lots of I/O we could and should rethink this strategy.

**The `IoConcurrentComputeGraph`**\
For this graph we take a different approach. We say that for some graphs with forking patterns:
```$xslt
            node_2
           /
          /
    node_1
          \
           \
            node_3  
```

`node_2` and `node_3` can be executed in different thread. 



### The Pipeline package

This is mostly me playing w rust. 
I wanted to build a simple MapReduce because it's probably the version_0 of any graph pipeline.
It was also a nice way to discover a few features in `rust`
