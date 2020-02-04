use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::sync::Arc;
use std::thread::{JoinHandle, spawn};


pub type MapLikeFunc = fn(xs: Vec<u8>) -> Vec<u8>;

pub struct MapLikeSeq {
    pub fs: Arc<Vec<MapLikeFunc>>
}

// todo need to make it work w thread pool
impl MapLikeSeq where {

    pub fn new(funcs: Vec<MapLikeFunc>) -> Self {
        return MapLikeSeq{fs: Arc::new(funcs)};
    }

    pub fn apply(&self, datum: Vec<u8>) -> Vec<u8> {
        let mut dat = datum.clone();
        let fs = Arc::clone(&self.fs);
        for i in 0..fs.len() {
            let f = fs[i];
            dat = f(dat);
        }
        return dat;
    }

    pub fn apply_async(&self, datum: Vec<u8>) -> JoinHandle<Vec<u8>> {
        let mut dat = datum.clone();
        let fs = Arc::clone(&self.fs);
        return spawn(move || {
            for i in 0..fs.len() {
                let f = fs[i];
                dat = f(dat);
            }
            return dat;
        });
    }

    pub fn map_async(&self, data: Vec<Vec<u8>>) -> Vec<JoinHandle<Vec<u8>>> {
        let mut handles = vec![];
        for datum in data {
            handles.push(self.apply_async(datum));
        }
        return handles;
    }

    pub fn compute_async(&self,
                     work_sender: Sender<Work>,
                     work_receiver: Receiver<Work>) -> (Sender<JoinHandle<Vec<u8>>>, Receiver<JoinHandle<Vec<u8>>>) {
        let (io_out_sender, io_out_receiver) = channel();
        loop {
            let result = work_receiver.recv_timeout(Duration::from_millis(100));
            if result.is_err() {
                break;
            }
            let work = result.unwrap();
            io_out_sender.send(self.apply_async(work.datum));
        }

        return (io_out_sender, io_out_receiver);
    }
}

pub struct Work {
    pub datum: Vec<u8>,
}
