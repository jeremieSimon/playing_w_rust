use std::collections::{VecDeque, HashMap};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;
use atomic_refcell;

pub use crate::graph::GraphLikeFunc;

// exposed graph structure
pub struct ConcurrentComputeGraph<T> where T: Clone {
    pub root: Arc<ConcurrentGraphNode<T>>,
    internal_root: Arc<ConcurrentInternalGraphNode<T>>,
}

impl <T> ConcurrentComputeGraph<T> where T: Clone {

    pub fn new(root: Arc<ConcurrentGraphNode<T>>) -> ConcurrentComputeGraph<T> {
        return ConcurrentComputeGraph {
            root: Arc::clone(&root),
            internal_root: ConcurrentInternalGraphNode::from(ConcurrentTmpInternalGraphNode::to_internal_graph_node(Arc::clone(&root))),
        }
    }

    fn apply(&self, datum: Vec<T>) -> Vec<T> {
        return self.internal_root.apply(datum);
    }

    pub fn apply_batch(&self, data: Vec<Vec<T>>) -> Vec<Vec<T>> {
        return self.internal_root.apply_batch(data);
    }
}

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

// *******************************
// internal concurrent graph repr.
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! NOTE: (README) !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// We have 2 internal repr of the graph, one mutable and one immutable.
// The mutable one is only used temporarly to allow us to transpose the graph that is user generated.
// The immutable one, is the one we keep in the structure of the compute graph.
// Having an immutable graph makes is a little bit faster when dealing with high concurrency.
// *******************************
type ConcurrentParentRefs<T> = Vec<Arc<ConcurrentInternalGraphNode<T>>>;

// for the internal structure each node points to its parents.
// because of the non-natural way to express such a graph, we keep this representation private.
pub struct ConcurrentInternalGraphNode<T> {
    f: GraphLikeFunc<T>,
    m: String,
    parents: ConcurrentParentRefs<T>,
    id: Uuid,
}

impl <T> ConcurrentInternalGraphNode <T> where T: Clone {

    fn from(sink_node: Arc<ConcurrentTmpInternalGraphNode<T>>) -> Arc<ConcurrentInternalGraphNode<T>> {
        if sink_node.parents.borrow().len() == 0 {
            return Arc::new(ConcurrentInternalGraphNode {
                f: sink_node.f,
                m: sink_node.m.clone(),
                id: sink_node.id,
                parents: vec![],
            });
        }

        let mut parents = Vec::new();
        for parent in sink_node.parents.borrow().iter() {
            let n = ConcurrentInternalGraphNode::from(Arc::clone(parent));
            parents.push(n);
        }

        return Arc::new(ConcurrentInternalGraphNode {
            f: sink_node.f,
            m: sink_node.m.clone(),
            id: sink_node.id,
            parents,
        });
    }

    // given the tap node, apply starting from sink node up to the tap.
    // to be used for concurrent application.
    fn apply(&self, datum: Vec<T>) -> Vec<T> {
        let f = self.f;
        let mut data = vec![];
        if self.parents.len() == 0 {
            return f(datum);
        }
        for parent in &self.parents{
            let result = parent.apply(datum.clone());
            data.extend(result);
        }
        return f(data);
    }

    // given the tap node, apply starting from sink node up to the tap.
    // to be used for concurrent application.
    fn apply_batch(&self, batch: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let f = self.f;
        let mut data: Vec<Vec<T>> = Vec::new();
        if self.parents.len() == 0 {
            return batch.iter().map(|xs| f(xs.to_vec())).collect();
        }
        for parent in &self.parents {
            let result = parent.apply_batch(batch.clone());
            data.extend(result);
        }
        return data.iter().map(|xs| f(xs.to_vec())).collect();
    }
}


// *******************************
// internal temporary concurrent graph repr, on which we allow mutation.
// *******************************
type ConcurrentParentMutablRefs<T> = atomic_refcell::AtomicRefCell<Vec<Arc<ConcurrentTmpInternalGraphNode<T>>>>;
// for the internal structure each node points to its parents.
// because of the non-natural way to express such a graph, we keep this representation private.
pub struct ConcurrentTmpInternalGraphNode<T> {
    f: GraphLikeFunc<T>,
    m: String,
    parents: ConcurrentParentMutablRefs<T>,
    id: Uuid,
}

impl <T> fmt::Display for ConcurrentTmpInternalGraphNode<T> where T: Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, m: {}", self.id, self.m)
    }
}

impl <T> ConcurrentTmpInternalGraphNode<T> where T: Clone {

    fn empty(f: GraphLikeFunc<T>, m: String, id: Uuid) -> ConcurrentTmpInternalGraphNode<T> {
        return ConcurrentTmpInternalGraphNode {
            f,
            m,
            id,
            parents: atomic_refcell::AtomicRefCell::new(vec![])
        };
    }
    // we start from the root node, and build a transpose of the given graph.
    // ref: https://en.wikipedia.org/wiki/Transpose_graph
    fn to_internal_graph_node(node: Arc<ConcurrentGraphNode<T>>) -> Arc<ConcurrentTmpInternalGraphNode<T>> {

        // 1. declare all necessary structures
        let mut nodes = VecDeque::new();
        let mut internal_nodes = VecDeque::new();
        let mut id_to_internal_node = HashMap::new();


        let internal = ConcurrentTmpInternalGraphNode::empty(node.f, node.m.clone(), node.id);

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
                let new_internal: Arc<ConcurrentTmpInternalGraphNode<T>> = {
                    if id_to_internal_node.contains_key(&child.id) {
                        Arc::clone(&id_to_internal_node.get(&child.id).unwrap())
                    } else {
                        Arc::new(ConcurrentTmpInternalGraphNode::empty(child.f, child.m.clone(), child.id))
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