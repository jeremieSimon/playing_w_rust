### Playing w/ rust

### Graph Package
In this package there are 3 different implementations of `ComputableGraph`: A normal (serial) implementation, and 2 concurrent ones (`ConcurrentComputeGraph`, `IoConcurrentComputeGraph`).

**Logic Flow**\
At a very high level, the flow is as follow: given a user defined graph (concurrent or not)
- We build a new internal representation so that instead of having the parent node pointing to their children, we have child nodes pointing to theirs respective parents.
 This makes the computation easier. 
 The concurrent graph is just an extension of the normal one except that we use `Arc` instead of `Rc`. 
- Both graph can work on a datum or a batch of data. 
- The IoGraph has more metadata attached to its node, to facilitate `Fork` and `Join` operations. 

### *Parallelism Design*
#
**The `ConcurrentComputeGraph`**\
For a single datum or batch of data, each node is ran within the same thread.  
The user of this API can then divide its data so that each datum, can run in a different thread.
The concurrent graph is therefore optimized for **throughput**. 
This approach has the following pros:
1. Have a perfect maximization of the CPU usage with minimal interrupts.\
If we're executing our code on a 4 core machines, we can have 4 batches running at the time. Every time a batch is completed, a Thread is taking the next batch to process from a queue. This process continues until there is no more data to process.

2. Performance is very predictable\
For applications with strict SLAs, predictability in performance is key. You want your performance to have very little variance.
If each datum is executed in a single thread, then approximating the performance of a single call to `ConcurrentComputeGraph.apply` should be about the same as approximating whether it is executed in mutli-threaded context or not. 

3. Optimizations\
If the performance is predictable, and the execution flow for a single datum is constently the same, then it becomes easier to optimize and profile an application.
We might want to say that some operations can be vectorized and use SIMD instructions.

All the above pros hold when the operators in a graph are CPU bounded. 
If we were to have operators with lots of I/O we could and should rethink this strategy.
#
**The `IoConcurrentComputeGraph`**\
For this graph we take a different approach. We say that for some graphs with forking patterns:\

**Figure 1.1**
```$xslt
            node_2
           /
          /
    node_1
          \
           \
            node_3  
```

`node_2` and `node_3` can be executed in different threads.
The motivation here is to say that if operators are I/O bounded, then having a single thread blocking until a syscall comes back is a waste of resources.

The code is broken into 2 pieces, we have a `scheduler` and some `executor`

The `scheduler` runs on the main thread. The `scehduler` comes up with an executable plan, and sends it to the executor. 
The `executor` is then executing the plan on a different thread.

We have an internal representation of the graph where we add some meta information to each node. 
We add 2 important piece of information. 
1. We say that a node is `Joinable` if it has multiple parents.\
In **Figure 1.2**, *node_7*, and *node_8* are both `joinable`.

2. We say that a node is `Forkable` if it has multiple children\
In **Figure 1.2**, *node_1*, and *node_4* are both `forkable`.

#
Let's see a what a more compelling graph could look like.\
**Figure 1.2**

```$xslt
              node_5
              /      \
             /        \
            /          \
        node_4        node_7
          / \        /       \
         /   \      /         \
        /     node_6           \
node_1 /                       node_8
       \                       /
        \                     /
         \                   /
         node_2 ----- node_3

``` 

In the example above, the execution plan will look like this.\
**Thread1**: `[node_1 -> node_2, node_2 -> node_3, node_3 -> node_8]`\
**Thread2**: `[node_1 -> node_4]`\
--------------------------------**Thread3**: `[node_4 -> node_5, node_5 -> node_7]`\
--------------------------------**Thread4**: `[node_4 -> node_6, node_6 -> node_7]`\
----------------------------------------------------------------------------------**Thread5**: `[node_7 -> node_8]`

Note that this not completely optimal because 
- you could argue that **Thread2** should fork directly which is not what's happening in the current implementation.
- the join strategy could be much smarter. \
For now, on join, we wait until all threads have completed. In a better impl we could have the scheduler to keep scheduling new `plan` while it waits for others to join. \
In the above example, it doesn't make much diff at there are not many joins, but on large graph it could bring huge gains, especially when dealing with long I/O nodes.
  

#
### The Pipeline package

This is mostly me playing w rust. 
I wanted to build a simple MapReduce because it's probably the version_0 of any graph pipeline.
It was also a nice way to discover a few features in `rust`
