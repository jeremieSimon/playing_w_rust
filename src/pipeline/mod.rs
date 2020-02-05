use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::Duration;
use std::sync::Arc;
use std::thread::{JoinHandle, spawn};
use std::collections::HashMap;
use std::hash::{Hash};
use std::borrow::Borrow;
use std::cell::{RefCell, Ref};

pub mod word_count;

// map stuff
pub type MapLikeFunc = fn(xs: Vec<u8>) -> Vec<u8>;

pub struct MapLikeSeq {
    pub fs: Arc<Vec<MapLikeFunc>>,
}

// for shuffle purpose
pub trait Keyable<K: Sized + Hash + Eq, V: ?Sized> {
    fn get_key(&self) -> K;
    fn get_value(&self) -> &V;
}



// todo need to make it work w thread pool
impl MapLikeSeq {

    pub fn new(funcs: Vec<MapLikeFunc>) -> Self {
        return MapLikeSeq{ fs: Arc::new(funcs) };
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
                     work_sender: Sender<Vec<u8>>,
                     work_receiver: Receiver<Vec<u8>>) -> (Sender<JoinHandle<Vec<u8>>>, Receiver<JoinHandle<Vec<u8>>>) {
        let (io_out_sender, io_out_receiver) = channel();
        loop {
            let result = work_receiver.recv_timeout(Duration::from_millis(100));
            if result.is_err() {
                break;
            }
            let work = result.unwrap();
            io_out_sender.send(self.apply_async(work));
        }

        return (io_out_sender, io_out_receiver);
    }
}


// pipeline stuff
pub type MapStageOutput<K, V> = Vec<Box<Keyable<K, V>>>;
pub type MmapLikeFunc<K, V> = fn(bytes: Vec<u8>) -> MapStageOutput<K, V>;
pub type ReduceLikeFunc<K, V> = fn(k: K, vs: Ref<Vec<V>>) -> (K, V);

pub struct PipelineStage<K, V> {
    pub map_func: MmapLikeFunc<K, V>,
    pub reduce_func: ReduceLikeFunc<K, V>,
}

impl<K: Sized + Hash + Eq + Clone, V: Clone> PipelineStage<K, V>{

    pub fn apply_map(&self, bytes: Vec<u8>) -> MapStageOutput<K, V> {
        let f = self.map_func;
        return f(bytes);
    }

    pub fn apply_shuffle(&self, map_outputs: MapStageOutput<K, V>) -> HashMap<K, RefCell<Vec<V>>> {
        let mut k_to_values: HashMap<K, RefCell<Vec<V>>> = HashMap::new();
        for keyable in map_outputs.iter() {
            let k = keyable.get_key();
            let element = keyable.get_value();
            if !k_to_values.contains_key(&k) {
                k_to_values.insert(k.clone(), RefCell::new(vec![]));
            }
            k_to_values.get(&k).unwrap().borrow_mut().push(element.clone());
        }

        return k_to_values;
    }

    pub fn apply_reduce(&self, grouped_by_key: HashMap<K, RefCell<Vec<V>>>) -> Vec<(K, V)> {
        let mut reduce_output = Vec::new();
        for (k, vs) in grouped_by_key.iter() {
            let f = self.reduce_func;
            let (new_k, new_v) = f(k.clone(), vs.borrow());
            reduce_output.push((new_k, new_v));
        }
        return reduce_output;
    }
}