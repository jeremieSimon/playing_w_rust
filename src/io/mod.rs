use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::prelude::*;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};

use std::time::Duration;
use std::thread::JoinHandle;

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

pub fn read_and_transform_to_work_unit<'a>(filename: String) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {

    let (io_sender, io_receiver) = read(filename).unwrap();
    let (work_sender, work_receiver) = channel();
    loop {
        let result = io_receiver.recv_timeout(Duration::from_millis(100));
        if result.is_err() {
            break;
        }
        let bytes = result.unwrap();
        work_sender.send(bytes);
    }
    return (work_sender, work_receiver);
}

// WRITE
pub fn write(filename: String, io_out_receiver: Receiver<JoinHandle<Vec<u8>>>) -> io::Result<()> {
    let f = File::create(filename)?;
    let mut buf_writer = BufWriter::new(f);
    loop {
        let result = io_out_receiver.recv_timeout(Duration::from_millis(100));
        if result.is_err() {
            break;
        }
        let val = result.unwrap().join().unwrap();

        //let val = result.join().unwrap();
        let as_string = std::str::from_utf8(&val).unwrap();
        buf_writer.write(&val).unwrap();
    }

    buf_writer.flush().unwrap();
    return Ok(());
}
