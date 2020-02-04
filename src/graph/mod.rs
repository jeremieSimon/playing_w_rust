use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{VecDeque, HashMap};
use std::fmt;
use uuid::Uuid;
use std::sync::Arc;


pub struct ComputeGraph<T> where T: Clone {
    pub root: Rc<SimpleGraphNode<T>>,
    internal_root: Rc<InternalGraphNode<T>>,
}

impl <T> ComputeGraph<T> where T: Clone {

    pub fn new(root: Rc<SimpleGraphNode<T>>) -> ComputeGraph<T> {
        return ComputeGraph {
            root: Rc::clone(&root),
            internal_root: InternalGraphNode::to_internal_graph_node(Rc::clone(&root)),
        };
    }

    pub fn apply(&self, datum: Vec<T>) -> Vec<T> {
        return self.internal_root.apply(datum);
    }
}

// exposed graph structure
pub type GraphLikeFunc<T> = fn (xs: Vec<T>) -> Vec<T>;

pub struct SimpleGraphNode<T> where T: Clone {
    pub f: GraphLikeFunc<T>,
    pub m: String,
    pub children: Vec<Rc<SimpleGraphNode<T>>>,
    pub id: Uuid,

}

impl <T> SimpleGraphNode<T> where T: Clone {

    pub fn new(f: GraphLikeFunc<T>, m: String, children: Vec<Rc<SimpleGraphNode<T>>>) -> Self {
        return SimpleGraphNode {
                f,
                m,
                children,
                id: Uuid::new_v4(),
        };
    }
}

// internal graph construct.
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
    fn to_internal_graph_node(node: Rc<SimpleGraphNode<T>>) -> Rc<InternalGraphNode<T>> {
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

    // recursively apply.
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
}

