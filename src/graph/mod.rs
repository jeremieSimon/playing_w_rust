use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{VecDeque, HashMap};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;
use atomic_refcell;
use std::borrow::BorrowMut;


pub mod concurrent;

pub type GraphLikeFunc<T> = fn (xs: Vec<T>) -> Vec<T>;

// exposed graph structure
pub struct ComputeGraph<T> where T: Clone {
    pub root: Rc<GraphNode<T>>,
    internal_root: Rc<InternalGraphNode<T>>,
}

impl <T> ComputeGraph<T> where T: Clone {

    pub fn new(root: Rc<GraphNode<T>>) -> ComputeGraph<T> {
        return ComputeGraph {
            root: Rc::clone(&root),
            internal_root: InternalGraphNode::to_internal_graph_node(Rc::clone(&root)),
        };
    }

    pub fn apply(&self, datum: Vec<T>) -> Vec<T> {
        return self.internal_root.apply(datum);
    }

    pub fn apply_batch(&self, data: Vec<Vec<T>>) -> Vec<Vec<T>> {
        return self.internal_root.apply_batch(data);
    }
}


// *****************
// graph node region
// *****************
pub struct GraphNode<T> where T: Clone {
    pub f: GraphLikeFunc<T>,
    pub m: String,
    pub children: Vec<Rc<GraphNode<T>>>,
    id: Uuid,

}

impl <T> GraphNode<T> where T: Clone {

    pub fn new(f: GraphLikeFunc<T>, m: String, children: Vec<Rc<GraphNode<T>>>) -> Self {
        return GraphNode {
                f,
                m,
                children,
                id: Uuid::new_v4(),
        };
    }
}

// ****************************
// internal graph constructs.
// ****************************
type ParentRefs<T> = RefCell<Vec<Rc<InternalGraphNode<T>>>>;

// for the internal structure each node points to its parents.
// because of the non-natural way to express such a graph, we keep this representation private.
pub struct InternalGraphNode<T> {
    f: GraphLikeFunc<T>,
    m: String,
    parents: ParentRefs<T>,
    id: Uuid,
}

impl <T> fmt::Display for InternalGraphNode<T> where T: Clone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, m: {}", self.id, self.m)
    }
}

impl <T> InternalGraphNode<T> where T: Clone {

    // we start from the root node, and build a transpose of the given graph.
    // ref: https://en.wikipedia.org/wiki/Transpose_graph
    fn to_internal_graph_node(node: Rc<GraphNode<T>>) -> Rc<InternalGraphNode<T>> {
        let mut nodes = VecDeque::new();
        let mut internal_nodes = VecDeque::new();
        let mut id_to_internal_node = HashMap::new();

        let internal = InternalGraphNode {
            f: node.f,
            m: node.m.clone(),
            id: node.id,
            parents: RefCell::new(vec![])
        };

        let mut internal_rc = Rc::new(internal);

        id_to_internal_node.insert(node.id, Rc::clone(&internal_rc));
        nodes.push_back(node);
        internal_nodes.push_back(Rc::clone(&internal_rc));

        while nodes.len() != 0 {
            let node = nodes.pop_front().unwrap();
            let internal_node = internal_nodes.pop_front().unwrap();

            for child in node.children.iter() {

                let new_internal: Rc<InternalGraphNode<T>> = {
                    if id_to_internal_node.contains_key(&child.id) {
                        Rc::clone(&id_to_internal_node.get(&child.id).unwrap())
                    } else {
                        Rc::new(InternalGraphNode {
                            f: child.f,
                            m: child.m.clone(),
                            id: child.id,
                            parents: RefCell::new(vec![])
                        })
                    }
                };
                new_internal.parents.borrow_mut().push(Rc::clone(&internal_node));
                id_to_internal_node.insert(new_internal.id, Rc::clone(&new_internal));

                internal_nodes.push_back(Rc::clone(&new_internal));
                nodes.push_back(Rc::clone(child));
                internal_rc = new_internal;
            }
        }
        return internal_rc;
    }

    // given the tap node, apply starting from sink node up to the tap.
    // fits for general purpose computation.
    fn apply(&self, datum: Vec<T>) -> Vec<T> {
        let f = self.f;
        let mut data = vec![];
        if self.parents.borrow().len() == 0 {
            return f(datum);
        }
        for parent in self.parents.borrow().iter() {
            let result = parent.apply(datum.clone());
            data.extend(result);
        }
        return f(data);
    }

    fn apply_batch(&self, batch: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let f = self.f;
        let mut data: Vec<Vec<T>> = Vec::new();
        if self.parents.borrow().len() == 0 {
            return batch.iter().map(|xs| f(xs.to_vec())).collect();
        }
        for parent in self.parents.borrow().iter() {
            let result = parent.apply_batch(batch.clone());
            data.extend(result);
        }
        return data.iter().map(|xs| f(xs.to_vec())).collect();
    }
}

