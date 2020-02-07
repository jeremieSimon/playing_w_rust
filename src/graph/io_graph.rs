use std::collections::{VecDeque, HashMap, HashSet};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;
use atomic_refcell;
use std::thread::{spawn, JoinHandle};


pub use crate::graph::GraphLikeFunc;
use crate::graph::concurrent::{ConcurrentGraphNode, ConcurrentInternalGraphNode, ConcurrentTmpInternalGraphNode};


//
// exposed graph structure
pub struct IoConcurrentComputeGraph<T> where T: Clone {
    pub root: Arc<ConcurrentGraphNode<T>>,
    internal_root: Arc<IoInternalGraphNode<T>>,
}

impl <T> IoConcurrentComputeGraph<T> where T: Clone + Send + Sync + Copy + fmt::Display + fmt::Debug + 'static  {
    pub fn new(root: Arc<ConcurrentGraphNode<T>>) -> IoConcurrentComputeGraph<T> {
        return IoConcurrentComputeGraph {
            root: Arc::clone(&root),
            internal_root: IoInternalGraphNode::from(Arc::clone(&root)),
        }
    }

    pub fn apply(&self, datum: Vec<T>) -> Vec<T> {
        return IoInternalGraphNode::schedule_bfs(Arc::clone(&self.internal_root), datum);
    }
}

type ConcurrentParentRefs<T> = atomic_refcell::AtomicRefCell<Vec<Arc<IoInternalGraphNode<T>>>>;

// for the internal structure each node points to its parents.
// because of the non-natural way to express such a graph, we keep this representation private.
#[derive(Debug)]
pub struct IoInternalGraphNode<T> {
    pub f: GraphLikeFunc<T>,
    pub name: String,
    pub children: ConcurrentParentRefs<T>,
    pub id: Uuid,
    pub n_parents: i32,
    pub forkable: bool,
}

impl <T> fmt::Display for IoInternalGraphNode<T> where T: Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, m: {}, n_parents: {}, forkable: {}", self.id, self.name, self.n_parents, self.forkable)
    }
}


impl <T> IoInternalGraphNode<T> where T: Clone + Send + Sync + Copy + fmt::Display + fmt::Debug + 'static {

    // ********************
    // Scheduling Region
    // ********************
    pub fn schedule_bfs(root: Arc<IoInternalGraphNode<T>>, datum: Vec<T>) -> Vec<T> {
        let mut nodes = HashMap::new();
        let mut bfs_q = VecDeque::new();
        let mut uuid_to_handles: HashMap<Uuid, Vec<JoinHandle<Vec<T>>>> = HashMap::new();
        let mut scheduled_nodes = HashSet::new();

        nodes.insert(root.id, Arc::clone(&root));
        bfs_q.push_back(Arc::clone(&root));

        let mut results = datum.clone();
        while bfs_q.len() != 0 {
            let node = bfs_q.pop_front().unwrap();

            // 1. if node is of type join, wait until all computation is done.
            if node.n_parents > 1 || node.forkable {
                let handle_opt = uuid_to_handles.get(&node.id);

                // check if all computation upstream has been scheduled
                if handle_opt.is_some() && (node.forkable || (node.n_parents > 1 && handle_opt.unwrap().len() as i32 == node.n_parents)) {
                    results.clear();
                    let remove = uuid_to_handles.remove(&node.id).unwrap();
                    for handle in remove {
                        let result = handle.join().unwrap();
                        results.extend(result);
                    }
                }
            }

            // ensure not to schedule a node twice
            if scheduled_nodes.contains(&node.id) {
                continue;
            } else {
                scheduled_nodes.insert(node.id);
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

    fn async_exec(name_and_funcs: Vec<(String, GraphLikeFunc<T>)>, datum: Vec<T>) -> JoinHandle<Vec<T>> {
        let datum_clone = datum.clone();
        return spawn(move || {
            let mut result = datum_clone.clone();
            let names = name_and_funcs.clone();
            for (name, f) in name_and_funcs {
                result = f(result.clone());
            }
            println!("exec plan {:?}, before{:?}, after{:?}", names, datum_clone, result);
            return result;
        });
    }

    fn to_exec_plan(node: Arc<IoInternalGraphNode<T>>) -> (Arc<IoInternalGraphNode<T>>, Vec<(String, GraphLikeFunc<T>)>) {

        let mut name_and_funcs: Vec<(String, GraphLikeFunc<T>)> = vec![];
        let mut last_node = Arc::clone(&node);
        let mut nodes = VecDeque::new();

        nodes.push_back(Arc::clone(&node));

        while nodes.len() != 0 {
            let node = nodes.pop_front().unwrap();

            name_and_funcs.push((node.name.clone(), node.f));
            last_node = Arc::clone(&node);

            if node.forkable || node.n_parents > 1 {
                break;
            }
            for child in node.children.borrow().iter() {
                nodes.push_back(Arc::clone(child));
            }
        }

        return (last_node , name_and_funcs)
    }

    // *******************************
    // Building Internal Struct Region
    // *******************************
    pub fn from(root: Arc<ConcurrentGraphNode<T>>) -> Arc<IoInternalGraphNode<T>> {

        let new_root = IoInternalGraphNode {
            f: root.f,
            name: root.name.clone(),
            id: root.id,
            n_parents: 0,
            forkable: root.children.len() > 1,
            children: atomic_refcell::AtomicRefCell::new(vec![]),
        };
        let arced_new_root = Arc::new(new_root );

        let concurrent_node = ConcurrentInternalGraphNode::from(ConcurrentTmpInternalGraphNode::to_internal_graph_node(Arc::clone(&root)));
        let id_to_concurrent_node = IoInternalGraphNode::index_by_id(Arc::clone(&concurrent_node));

        let mut bfs_q = VecDeque::new();
        let mut io_nodes_bfs_q = VecDeque::new();
        let mut id_to_io_node: HashMap<Uuid, Arc<IoInternalGraphNode<T>>> = HashMap::new();
        let mut visited_nodes: HashSet<Uuid> = HashSet::new();

        bfs_q.push_back(Arc::clone(&root));
        io_nodes_bfs_q.push_back(Arc::clone(&arced_new_root));
        id_to_io_node.insert(arced_new_root.id, Arc::clone(&arced_new_root));

        while bfs_q.len() != 0 {
            let node = bfs_q.pop_front().unwrap();
            let io_node = io_nodes_bfs_q.pop_front().unwrap();

            for child in node.children.iter() {

                // 1. update number of parents.
                let n_parents = id_to_concurrent_node.get(&child.id).unwrap().parents.len() as i32;

                // 2. go fetch or create the io node
                let new_io_child = {
                    if !id_to_io_node.contains_key(&child.id) {
                        Arc::new(IoInternalGraphNode {
                            f: child.f,
                            name: child.name.clone(),
                            id: child.id,
                            n_parents,
                            forkable: child.children.len() > 1,
                            children: atomic_refcell::AtomicRefCell::new(vec![]),
                        })
                    } else {
                        Arc::clone(&id_to_io_node.get(&child.id).unwrap())
                    }
                };

                id_to_io_node.insert(child.id, Arc::clone(&new_io_child));

                // 3. update refs
                io_node.children.borrow_mut().push(Arc::clone(&new_io_child));
                if !visited_nodes.contains(&child.id) {
                    bfs_q.push_back(Arc::clone(child));
                    io_nodes_bfs_q.push_back(Arc::clone(&new_io_child));
                }
                visited_nodes.insert(child.id);
            }
        }

        return Arc::clone(&arced_new_root);
    }

    fn index_by_id(node: Arc<ConcurrentInternalGraphNode<T>>) -> HashMap<Uuid, Arc<ConcurrentInternalGraphNode<T>>> {
        let mut id_to_node = HashMap::new();
        let mut bfs_q = VecDeque::new();
        bfs_q.push_back(Arc::clone(&node));

        while bfs_q.len() != 0 {
            let node = bfs_q.pop_front().unwrap();
            id_to_node.insert(node.id, Arc::clone(&node));

            for child in node.parents.iter() {
                bfs_q.push_back(Arc::clone(child));
            }
        }
        return id_to_node;
    }
}