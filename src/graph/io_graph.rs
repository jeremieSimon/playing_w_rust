use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::{VecDeque, HashMap};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;
use atomic_refcell;

pub use crate::graph::GraphLikeFunc;
use std::thread::{spawn, JoinHandle};
use std::ptr::hash;
use std::cell::RefCell;


type ConcurrentParentRefs<T> = atomic_refcell::AtomicRefCell<Vec<Arc<IoInternalGraphNode<T>>>>;

// for the internal structure each node points to its parents.
// because of the non-natural way to express such a graph, we keep this representation private.
pub struct IoInternalGraphNode<T> {
    pub f: GraphLikeFunc<T>,
    pub name: String,
    pub children: ConcurrentParentRefs<T>,
    pub id: Uuid,
    pub joinable: i32,
}

impl <T> IoInternalGraphNode<T> where T: Clone + Send + Sync + Copy + fmt::Display + fmt::Debug + 'static {

    pub fn schedule_bfs(root: Arc<IoInternalGraphNode<T>>, datum: Vec<T>) -> Vec<T> {
        let mut nodes = HashMap::new();
        let mut bfs_q = VecDeque::new();
        let mut uuid_to_handles: HashMap<Uuid, Vec<JoinHandle<Vec<T>>>> = HashMap::new();

        nodes.insert(root.id, Arc::clone(&root));
        bfs_q.push_back(Arc::clone(&root));

        let mut results = datum.clone();
        while bfs_q.len() != 0 {
            let node = bfs_q.pop_front().unwrap();
            println!("poped node {} -> {}", node.name, node.id);

            // 1. if node is of type join, wait until all computation is done.
            if node.joinable != 0 {
                let handle_opt = uuid_to_handles.get(&node.id);
                // all computation upstream has been scheduled
                if handle_opt.is_some() && handle_opt.unwrap().len() as i32 == node.joinable {
                    results.clear();
                    let remove = uuid_to_handles.remove(&node.id).unwrap();
                    for handle in remove {
                        let result = handle.join().unwrap();
                        results.extend(result);
                    }
                }
            }

            // 2. fork if possible and propagate the result downstream.
            for child in node.children.borrow().iter() {
                let (last_node, exec_plan) = IoInternalGraphNode::to_exec_plan(Arc::clone(&child));
                let handle = IoInternalGraphNode::async_exec(exec_plan, results.clone());
                let removed = uuid_to_handles.remove(&last_node.id);
                if removed.is_none() {
                    uuid_to_handles.insert(last_node.id, vec![handle]);
                } else {
                    let mut handles = removed.unwrap();
                    handles.push(handle);
                    uuid_to_handles.insert(last_node.id, handles);
                }
                bfs_q.push_back(Arc::clone(&last_node));
            }

        }
        return results;
    }

    pub fn async_exec(name_and_funcs: Vec<(String, GraphLikeFunc<T>)>, datum: Vec<T>) -> JoinHandle<Vec<T>> {
        let datum_clone = datum.clone();
        return spawn(move || {
            let mut result = datum_clone;
            let names = name_and_funcs.clone();
            for (name, f) in name_and_funcs {
                result = f(result.clone());
            }
            println!("exec plan {:?}, plan{:?}", names, result);
            return result;
        });
    }

    pub fn to_exec_plan(node: Arc<IoInternalGraphNode<T>>) -> (Arc<IoInternalGraphNode<T>>, Vec<(String, GraphLikeFunc<T>)>) {
        let mut name_and_funcs: Vec<(String, GraphLikeFunc<T>)> = vec![(node.name.clone(), node.f)];
        let mut last_node = Arc::clone(&node);
        for child in node.children.borrow().iter() {
            name_and_funcs.push((child.name.clone(), child.f));
            last_node = Arc::clone(&child);
            if child.forkable || child.joinable != 0 {
                break;
            }
        }
        return (last_node , name_and_funcs)
    }

    pub fn simple_exec(root: Arc<IoInternalGraphNode<T>>, datum: Vec<T>) -> Vec<T> {

        let mut node_to_async_datum: HashMap<Uuid, Vec<JoinHandle<Vec<T>>>> = HashMap::new();
        let mut handles: Vec<JoinHandle<Vec<T>>> = Vec::new();

        if root.forkable {
            for child in root.children.borrow().iter() {
                let f = child.f;
                let cloned = datum.clone();
                let handle = spawn(move || {
                    return f(cloned);
                });

                let mut removed = node_to_async_datum.remove(&child.id);

                if removed.is_none() {
                    let handles = vec![handle];
                    node_to_async_datum.insert(child.id, handles);
                } else {
                    let mut handles = removed.unwrap();
                    handles.push(handle);
                    node_to_async_datum.insert(child.id, handles);
                }
            }
        }

        let mut results = vec![];

        for child in root.children.borrow().iter() {
            for handle in node_to_async_datum.remove(&child.id).unwrap() {
                let result = handle.join().unwrap();
                results.extend(result);
            }
        }

        return results;
    }
}