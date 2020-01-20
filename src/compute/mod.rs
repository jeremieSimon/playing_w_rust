use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;


type MapLikeFunc = fn(xs: Vec<u8>) -> Vec<u8>;


pub struct Work<'a> {
    pub datum: Vec<u8>,
    pub compute_node: &'a ComputeNode<'a>
}

pub struct ComputeNode<'a> {
    pub curr: MapLikeFunc,
    pub next: Box<Option<&'a ComputeNode<'a>>>
}

pub fn compute<'a>(work_sender: Sender<Work<'a>>,
                  work_receiver: Receiver<Work<'a>>) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>){

    let (io_out_sender, io_out_receiver) = channel();
    loop {
        let result = work_receiver.recv_timeout(Duration::from_millis(100));
        if result.is_err() {
            break;
        }
        // compute the work
        let work = result.unwrap();
        let f = work.compute_node.curr;
        let val = f(work.datum);

        // if more work, then push next task into queue
        // else push into receiver queue
        if work.compute_node.next.is_some() {
            match work.compute_node.next.as_ref() {
                Some(x) => {
                    let w = Work {
                        datum: val,
                        compute_node: x
                    };
                    work_sender.send(w);
                    {}
                }
                _ => {}
            }
        } else {
            io_out_sender.send(val);
        }
    }

    return (io_out_sender, io_out_receiver);
}