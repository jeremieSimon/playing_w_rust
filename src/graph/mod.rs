use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::Borrow;
use std::collections::{VecDeque, HashSet, HashMap};
use std::fmt;

static mut COUNTER: i32 = 0;
pub struct ComputeGraph {
    pub root: Rc<SimpleGraphNode>,
    internal_root: Rc<InternalGraphNode>,
}

impl ComputeGraph {

    pub fn new(root: Rc<SimpleGraphNode>) -> ComputeGraph {
        return ComputeGraph {
            root: Rc::clone(&root),
            internal_root: InternalGraphNode::to_internal_graph_node(Rc::clone(&root)),
        };
    }

    pub fn apply(&self, datum: Vec<f64>) -> Vec<f64> {
        return self.internal_root.apply(datum);
    }
}

pub type GraphLikeFunc = fn (xs: Vec<f64>) -> Vec<f64>;

pub struct SimpleGraphNode {
    pub f: GraphLikeFunc,
    pub m: String,
    pub children: Vec<Rc<SimpleGraphNode>>,
    pub id: i32,

}
impl SimpleGraphNode {

    pub fn new(f: GraphLikeFunc, m: String, children: Vec<Rc<SimpleGraphNode>>) -> Self {
        unsafe {
            COUNTER += 1;
            return SimpleGraphNode {
                f,
                m,
                children,
                id: COUNTER,
            };
        }
    }
}


type parent_refs = RefCell<Vec<Rc<InternalGraphNode>>>;

pub struct InternalGraphNode {
    f: GraphLikeFunc,
    m: String,
    parents: parent_refs,
    id: i32,
}

impl fmt::Display for InternalGraphNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "id: {}, m: {}", self.id, self.m)
    }
}

impl InternalGraphNode {

    fn to_internal_graph_node(node: Rc<SimpleGraphNode>) -> Rc<InternalGraphNode> {
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

                let new_internal: Rc<InternalGraphNode> = {
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
    fn apply(&self, datum: Vec<f64>) -> Vec<f64> {
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
