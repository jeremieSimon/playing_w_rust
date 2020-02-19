use std::sync::Arc;
use std::rc::Rc;
use std::thread::{spawn, sleep};

use crate::graph::serial::{GraphNode, ComputeGraph};
use crate::graph::easy_functions;
use crate::graph::io_graph;
use crate::graph::concurrent::{ConcurrentGraphNode, ConcurrentComputeGraph};
use crate::pipeline::word_count::WordCount;
use std::collections::HashMap;
use std::borrow::BorrowMut;
use std::env::var;
use std::cell::RefCell;
use uuid::Uuid;
use atomic_refcell;
use std::time::Duration;
use crate::future::PollableFuture;

extern crate serde;
extern crate serde_json;

mod pipeline;
mod stupid_work;
mod io;
mod graph;
mod future;

fn main() {
    println!("--- playing w graph");
    graph_example();

    println!("--- playing w concurrent graph");
    concurrent_graph_example();

    println!("--- playing w io concurrent graph");
    concurrent_io_graph();

    println!("--- playing w a par map example");
    par_map_example();

    println!("--- playing w a simple map reduce");
    word_count();
}

fn par_map_example() {
    let filename_in = String::from("data/data.txt");
    let filename_out = String::from("data/data_out.txt");

    // example of the most simple graph possible
    let map_like_seq = pipeline::par_map::MapLikeSeq::new(vec![
        stupid_work::raw_to_text,
        stupid_work::text_to_tokens,
        stupid_work::tokens_to_json]);

    // open io in
    let (work_sender, work_receiver) = io::read_and_transform_to_work_unit(filename_in);
    // open compute
    let (io_out_sender, io_out_receiver) = map_like_seq.compute_async(work_sender, work_receiver);
    // io out:
    io::write(filename_out, io_out_receiver);
}

fn concurrent_graph_example() {
    //                      SIMPLE GRAPH EXAMPLE
    //
    //                        add_one node (mid node 1) \
    //                   /                               \
    //                  /
    // add_one node (starting node)                         square node (last node)
    //                  \                                 /
    //                   \  add_five node (mid node 2)   /
    //
    //

    let last_node = Arc::new(ConcurrentGraphNode::new(easy_functions::square,
                                                  String::from("last node"),
                                                  vec![]));
    let mid_node1 = Arc::new(ConcurrentGraphNode::new(easy_functions::add_one,
                                                  String::from("mid node 1"),
                                                  vec![Arc::clone(&last_node)]));
    let mid_node2 = Arc::new(ConcurrentGraphNode::new(easy_functions::add_five,
                                                  String::from("mid node 2"),
                                                  vec![Arc::clone(&last_node)]));
    let start_node = Arc::new(ConcurrentGraphNode::new(easy_functions::add_one,
                                                   String::from("start node"),
                                                   vec![Arc::clone(&mid_node1), Arc::clone(&mid_node2)]));

    let concurrent_graph = ConcurrentComputeGraph::new(Arc::clone(&start_node));

    let handle = spawn(move || {
        return concurrent_graph.apply_batch(vec![vec![1.0, 2.0], vec![5.0, 5.0]]);
    });
    let results = handle.join().unwrap();
    println!("batch mode: {:?}", results)
}

fn graph_example() {
    //                      SIMPLE GRAPH EXAMPLE
    //
    //                       add_one node (mid node 1) \
    //                   /                              \
    //                  /
    // add_one node (starting node)                     square node (last node)
    //                  \                                /
    //                   \  add_one node (mid node 2)   /
    //
    //
    let last_node = Rc::new(GraphNode::new(easy_functions::square,
                                                  String::from("last node"),
                                                  vec![]));
    let mid_node1 = Rc::new(GraphNode::new(easy_functions::add_one,
                                                  String::from("mid node 1"),
                                                  vec![Rc::clone(&last_node)]));
    let mid_node2 = Rc::new(GraphNode::new(easy_functions::add_one,
                                                  String::from("mid node 2"),
                                                  vec![Rc::clone(&last_node)]));
    let start_node = Rc::new(GraphNode::new(easy_functions::add_one,
                                                   String::from("start node"),
                                                   vec![Rc::clone(&mid_node1), Rc::clone(&mid_node2)]));

    let compute_graph = ComputeGraph::new(start_node);
    let applied_all = compute_graph.apply(vec![1.0, 2.0]);
    println!("single mode {:?}", applied_all);
    let batch_result = compute_graph.apply_batch(vec![vec![1.0, 2.0]]);
    println!("batch mode {:?}", batch_result);
}

fn word_count() {
    let line = String::from("hello world yo universe hello yp yo yop");
    let bytes = line.as_bytes().to_vec();
    let word_count_pipeline = pipeline::map_reduce::MapReduce {
        map_func: pipeline::word_count::word_count_mapper,
        reduce_func: pipeline::word_count::word_count_reducer,
    };

    let reduce_output = word_count_pipeline.apply(bytes);
    for (k, v) in reduce_output.iter() {
        println!("{}, {}", k, v);
    }
}

fn concurrent_io_graph() {


    /*
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

    */

    let node8 = Arc::new(graph::concurrent::ConcurrentGraphNode::new(
        easy_functions::add_one,
        String::from("node 8"),
        vec![],
    ));

    let mid_node7 = Arc::new(
        graph::concurrent::ConcurrentGraphNode::new(
            easy_functions::add_one,
            String::from("node 7"),
            vec![Arc::clone(&node8)],
        ));

    let mid_node3 = Arc::new(
        graph::concurrent::ConcurrentGraphNode::new(
            easy_functions::add_one,
            String::from("node 3"),
            vec![Arc::clone(&node8)],
        ));

    let mid_node5 = Arc::new(
        graph::concurrent::ConcurrentGraphNode::new(
            easy_functions::add_one,
            String::from("node 5"),
            vec![Arc::clone(&mid_node7)],
            ));

    let mid_node6 = Arc::new(
        graph::concurrent::ConcurrentGraphNode::new(
            easy_functions::add_one,
            String::from("node 6"),
            vec![Arc::clone(&mid_node7)],
            )
    );

    let mid_node4 = Arc::new(
        graph::concurrent::ConcurrentGraphNode::new(
            easy_functions::add_one,
            String::from("node 4"),
            vec![Arc::clone(&mid_node5), Arc::clone(&mid_node6)],
            )
    );

    let mid_node2 = Arc::new(
        graph::concurrent::ConcurrentGraphNode ::new(
            easy_functions::add_one,
            String::from("node 2"),
            vec![Arc::clone(&mid_node3)],
            )
    );

    let node1 = Arc::new(
        graph::concurrent::ConcurrentGraphNode::new(
            easy_functions::add_one,
            String::from("node 1"),
            vec![Arc::clone(&mid_node2), Arc::clone(&mid_node4)])
    );

    let computable_graph = io_graph::IoConcurrentComputeGraph::new(Arc::clone(&node1));
    let results = computable_graph.apply(vec![1.0, 2.0]);
    println!("{:?}", results);
}