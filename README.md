###Playing w rust
The main piece is in the graph package. 
Given a user defined graph (concurrent or not)
 
- We build a new internal representation so that instead of having the parent node pointing to the children, we have child nodes pointing to theirs respective parents.
 This makes the computation easier. 
 The concurrent graph is just an extension of the normal one except that the we `Arc` instead of `Rc`. 
- Both graph can work on a datum or a batch of data. 

The rest, is mostly me playing w rust. 
I wanted to build a simple MapReduce because it's probably the version 0 of any graph pipeline.
