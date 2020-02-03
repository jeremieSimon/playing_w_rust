use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::panic::resume_unwind;
use std::sync::Arc;

extern crate serde;
extern crate serde_json;

mod compute;
mod stupid_work;
mod io;

fn main() {

    let filename_in = String::from("/Users/jeremiesimon/Desktop/coucou");
    let filename_out = String::from("/Users/jeremiesimon/Desktop/coucou2");

    // example of the most simple graph possible
    let map_like_seq = compute::MapLikeSeq::new(vec![
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
