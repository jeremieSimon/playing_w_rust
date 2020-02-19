use std::task::Poll;
use std::thread::{spawn, sleep};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use atomic_refcell::AtomicRefCell;
use std::ops::{DerefMut, Deref};
use serde::export::fmt::Debug;
use std::borrow::BorrowMut;


pub struct Future<T> {
    value: Mutex<Option<T>>,
    value_polled: Mutex<bool>,
    value_published: Mutex<bool>,
}

impl <T> Future<T> {
    pub fn new() -> Future<T> {
        Future {
            value: Mutex::new(None),
            value_polled: Mutex::new(false),
            value_published: Mutex::new(false)
        }
    }
}

pub trait PollableFuture<T> {
    fn busy_poll(&self) -> Option<T>;
    fn get(&self) -> Option<T>;
}

impl <T> PollableFuture<T> for Future<T> where T: Debug {
    fn busy_poll(&self) -> Option<T> {

        loop {
            if let r = self.get() {
                if r.is_some() {
                    return r;
                }
            }
            sleep(Duration::from_millis(100));
        }
    }

    fn get(&self) -> Option<T> {

        if !(*self.value_published.lock().unwrap()) {
            return None;
        }
        let t = self.value.lock().unwrap().take().unwrap();
        let mut state = self.value_polled.lock().unwrap();
        *state = true;
        return Some(t);
    }
}

pub trait WritableFuture<T> {
    fn put(&self, t: T);
}

impl <T> WritableFuture<T> for Future<T> {
    fn put(&self, t: T) {
        if *self.value_published.lock().unwrap() {
            return;
        }
        let mut value_state = self.value.lock().unwrap();
        *value_state = Some(t);

        let mut write_state = self.value_published.lock().unwrap();
        *write_state = true;
    }
}

pub fn yo() -> Arc<impl PollableFuture<String>> {
    let f = Arc::new(Future::new());
    let f2 = Arc::clone(&f);
    spawn (move || {
        sleep(Duration::from_millis(2000));
        f.put(String::from("yo"));

    });
    return f2;
}
