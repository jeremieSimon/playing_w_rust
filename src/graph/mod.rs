use std::borrow::BorrowMut;


pub mod concurrent;
pub mod serial;
pub mod easy_functions;

pub type GraphLikeFunc<T> = fn (xs: Vec<T>) -> Vec<T>;
