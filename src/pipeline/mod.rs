pub mod word_count;
pub mod par_map;
pub mod map_reduce;

// map stuff
pub type MapLikeFunc = fn(xs: Vec<u8>) -> Vec<u8>;
