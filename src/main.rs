use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::panic::resume_unwind;

extern crate serde;
extern crate serde_json;

mod compute;
mod stupid_work;
mod io;

fn main() {

    let filename_in = String::from("/Users/jeremiesimon/Desktop/coucou");

    // example of the most simple graph possible
    let compute_node_3 = compute::ComputeNode{
        curr: stupid_work::tokens_to_json,
        next: Box::new(None)
    };
    let compute_node_2 = compute::ComputeNode{
        curr: stupid_work::text_to_tokens,
        next: Box::new(Some(&compute_node_3))
    };
    let compute_node_1 = compute::ComputeNode{
        curr: stupid_work::raw_to_text,
        next: Box::new(Some(&compute_node_2))
    };

    // open io in
    let (work_sender, work_receiver) = io::read_and_transform_to_work_unit(filename_in, &compute_node_1);
    // open compute
    let (io_out_sender, io_out_receiver) = compute::compute(work_sender, work_receiver);

    // io out:
    loop {
        let result = io_out_receiver.recv_timeout(Duration::from_millis(100));
        if result.is_err() {
            break;
        }
        let val = result.unwrap();
        let as_string = std::str::from_utf8(&val).unwrap();
        let deserialized: stupid_work::TextAsTokens = serde_json::from_str(as_string).unwrap();
        println!("deserialized = {:?}", deserialized);
        println!("deserialized2 = {:?}", as_string);
    }

}
