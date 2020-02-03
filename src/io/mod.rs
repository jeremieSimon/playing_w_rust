use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::prelude::*;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

use crate::compute;
use std::time::Duration;

// READ
pub fn read(filename: String) -> io::Result<(Sender<Vec<u8>>, Receiver<Vec<u8>>)> {

    let (io_sender, io_receiver) = channel();

    let f = File::open(filename)?;
    let buf_reader = BufReader::new(f);

    for line in buf_reader.lines() {
        io_sender.send(line.unwrap().as_bytes().to_owned());
    }
    Ok(((io_sender, io_receiver)))
}

pub fn read_and_transform_to_work_unit<'a>(filename: String) -> (Sender<compute::Work>, Receiver<compute::Work>) {

    let (io_sender, io_receiver) = read(filename).unwrap();
    let (work_sender, work_receiver) = channel();
    loop {
        let result = io_receiver.recv_timeout(Duration::from_millis(100));
        if result.is_err() {
            break;
        }
        let bytes = result.unwrap();
        let work = compute::Work{datum: bytes};
        work_sender.send(work);
    }
    return (work_sender, work_receiver);
}


// WRITE
pub fn write(filename: String) {
    // todo
}




