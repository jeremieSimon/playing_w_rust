

pub use crate::pipeline::Keyable;
use std::cell::{RefCell, Ref};
use std::borrow::{BorrowMut, Borrow};


pub struct WordCount {
    word: String,
    count: i32,
}

impl Keyable<String, i32> for WordCount {
    fn get_key(&self) -> String {
        return self.word.clone();
    }

    fn get_value(&self) -> &i32 {
        return &self.count;
    }
}

pub fn word_count_mapper(raw: Vec<u8>) -> Vec<Box<Keyable<String, i32>>> {
    // 1. turn into text
    let full_text = std::str::from_utf8(&raw).unwrap();

    // 2. turn into tokens
    let mut word_counts: Vec<Box<Keyable<String, i32>>> = vec![];
    for token in full_text.to_lowercase().split(" ") {
        word_counts.push(Box::new(WordCount{word: String::from(token), count: 1}));
    }

    // 3.
    return word_counts;
}

pub fn word_count_reducer(key: String, vs: Ref<Vec<i32>>) -> (String, i32) {
    let mut sum = vs.iter().sum();
    return (key, sum);
}