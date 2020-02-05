use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{VecDeque, HashMap};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;
use atomic_refcell;
use std::borrow::BorrowMut;


pub mod concurrent;
pub mod serial;

pub type GraphLikeFunc<T> = fn (xs: Vec<T>) -> Vec<T>;
