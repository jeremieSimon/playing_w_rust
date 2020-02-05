use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{VecDeque, HashMap};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;
use atomic_refcell;
use std::borrow::BorrowMut;

pub use crate::graph::GraphLikeFunc;

// ****************************
// concurrent graph node region
// ****************************
pub struct ConcurrentGraphNode<T> where T: Clone {
    pub f: GraphLikeFunc<T>,
    pub m: String,
    pub children: Vec<Arc<ConcurrentGraphNode<T>>>,
    id: Uuid,
}

impl <T> ConcurrentGraphNode<T> where T: Clone {

    pub fn new(f: GraphLikeFunc<T>, m: String, children: Vec<Arc<ConcurrentGraphNode<T>>>) -> Self {
        return ConcurrentGraphNode {
            f,
            m,
            children,
            id: Uuid::new_v4(),
        };
    }
    pub fn empty(f: GraphLikeFunc<T>, m: String) -> Self {
        return ConcurrentGraphNode {
            f,
            m,
            id: Uuid::new_v4(),
            children: vec![],
        };
    }
}

// ****************************
// internal concurrent graph constructs.
// ****************************
// here, we're using atomic_refcell instead of RwLock because
type ConcurrentParentRefs<T> = atomic_refcell::AtomicRefCell<Vec<Arc<ConcurrentInternalGraphNode<T>>>>;

// for the internal structure each node points to its parents.
// because of the non-natural way to express such a graph, we keep this representation private.
pub struct ConcurrentInternalGraphNode<T> {
    f: GraphLikeFunc<T>,
    m: String,
    parents: ConcurrentParentRefs<T>,
    id: Uuid,
}

impl <T> fmt::Display for ConcurrentInternalGraphNode<T> where T: Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, m: {}", self.id, self.m)
    }
}

impl <T> ConcurrentInternalGraphNode<T> where T: Clone {
    // we start from the root node, and build a transpose of the given graph.
    // ref: https://en.wikipedia.org/wiki/Transpose_graph
    pub fn to_internal_graph_node(node: Arc<ConcurrentGraphNode<T>>) -> Arc<ConcurrentInternalGraphNode<T>> {

        // 1. declare all necessary structures
        let mut nodes = VecDeque::new();
        let mut internal_nodes = VecDeque::new();
        let mut id_to_internal_node = HashMap::new();

        let internal = ConcurrentInternalGraphNode {
            f: node.f,
            m: node.m.clone(),
            id: node.id,
            parents: atomic_refcell::AtomicRefCell::new(vec![])
        };

        let mut internal_arc = Arc::new(internal);

        // 2. initiate structure
        id_to_internal_node.insert(node.id, Arc::clone(&internal_arc));
        nodes.push_back(node);
        internal_nodes.push_back(Arc::clone(&internal_arc));

        // 3. iterate over graph bfs style.
        while nodes.len() != 0 {
            let node = nodes.pop_front().unwrap();
            let internal_node = internal_nodes.pop_front().unwrap();

            for child in node.children.iter() {

                // either get back already built node if it exists or create a new one.
                let new_internal: Arc<ConcurrentInternalGraphNode<T>> = {
                    if id_to_internal_node.contains_key(&child.id) {
                        Arc::clone(&id_to_internal_node.get(&child.id).unwrap())
                    } else {
                        Arc::new(ConcurrentInternalGraphNode {
                            f: child.f,
                            m: child.m.clone(),
                            id: child.id,
                            parents: atomic_refcell::AtomicRefCell::new(vec![])
                        })
                    }
                };
                new_internal.parents.borrow_mut().push(Arc::clone(&internal_node));
                id_to_internal_node.insert(new_internal.id, Arc::clone(&new_internal));

                internal_nodes.push_back(Arc::clone(&new_internal));
                nodes.push_back(Arc::clone(child));
                internal_arc = new_internal;
            }
        }

        return internal_arc;
    }
}