use std::sync::Arc;
use std::rc::Rc;
use std::thread::spawn;

use crate::graph::serial::{GraphNode, ComputeGraph};
use crate::graph::easy_functions;
use crate::graph::concurrent::{ConcurrentGraphNode, ConcurrentComputeGraph};
use crate::pipeline::word_count::WordCount;

extern crate serde;
extern crate serde_json;

mod pipeline;
mod stupid_work;
mod io;
mod graph;

fn main() {
    println!("--- playing w graph");
    graph_example();

    println!("--- playing w concurrent graph");
    concurrent_graph_example();

    println!("--- playing w a par map example");
    par_map_example();

    println!("--- playing w a simple map reduce");
    word_count();
}

fn par_map_example() {
    let filename_in = String::from("/Users/jeremiesimon/Desktop/coucou");
    let filename_out = String::from("/Users/jeremiesimon/Desktop/coucou2");

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