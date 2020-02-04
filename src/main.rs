use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::panic::resume_unwind;
use std::sync::Arc;
use std::rc::Rc;

extern crate serde;
extern crate serde_json;

mod pipeline;
mod stupid_work;
mod io;
mod graph;

fn main() {
    graph_example();
}

fn pipe_example() {
    let filename_in = String::from("/Users/jeremiesimon/Desktop/coucou");
    let filename_out = String::from("/Users/jeremiesimon/Desktop/coucou2");

    // example of the most simple graph possible
    let map_like_seq = pipeline::MapLikeSeq::new(vec![
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

fn graph_example() {
    let last_node = Rc::new(graph::SimpleGraphNode::new(stupid_work::square,
                                                        String::from("last node"),
                                                        vec![]));
    let mid_node1 = Rc::new(graph::SimpleGraphNode::new(stupid_work::add_one,
                                                        String::from("mid node 1"),
                                                        vec![Rc::clone(&last_node)]));
    let mid_node2 = Rc::new(graph::SimpleGraphNode::new(stupid_work::add_one,
                                                        String::from("mid node 2"),
                                                        vec![Rc::clone(&last_node)]));
    let start_node = Rc::new(graph::SimpleGraphNode::new(stupid_work::add_one,
                                                         String::from("start node"),
                                                         vec![Rc::clone(&mid_node1), Rc::clone(&mid_node2)]));

    let compute_graph = graph::ComputeGraph::new(start_node);
    let applied_all = compute_graph.apply(vec![1.0, 2.0]);
    println!("{:?}", applied_all)
}
