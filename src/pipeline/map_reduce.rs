use std::collections::HashMap;
use std::hash::{Hash};
use std::cell::{RefCell, Ref};


// map stuff
pub type MapStageOutput<K, V> = Vec<Box<Keyable<K, V>>>;
pub type MmapLikeFunc<K, V> = fn(bytes: Vec<u8>) -> MapStageOutput<K, V>;

// for shuffle purpose
pub trait Keyable<K: Sized + Hash + Eq, V: ?Sized> {
    fn get_key(&self) -> K;
    fn get_value(&self) -> &V;
}

// reduce stuff
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