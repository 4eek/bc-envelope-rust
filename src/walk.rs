use std::rc::Rc;

use crate::Envelope;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EdgeType {
    None,
    Subject,
    Assertion,
    Predicate,
    Object,
    Wrapped,
}

impl EdgeType {
    pub fn label(&self) -> Option<&'static str> {
        match self {
            EdgeType::Subject | EdgeType::Wrapped => Some("subj"),
            EdgeType::Predicate => Some("pred"),
            EdgeType::Object => Some("obj"),
            _ => None,
        }
    }
}

pub type Visitor<'a, Parent> = dyn Fn(Rc<Envelope>, usize, EdgeType, Parent) -> Parent + 'a;

impl Envelope {
    pub fn walk<Parent: Default + Clone>(self: Rc<Self>, hide_nodes: bool, visit: &Visitor<Parent>) {
        if hide_nodes {
            self.walk_tree(visit);
        } else {
            self.walk_structure(visit);
        }
    }

    fn walk_structure<Parent: Default + Clone>(self: Rc<Self>, visit: &Visitor<Parent>) {
        self._walk_structure(0, EdgeType::None, Default::default(), visit);
    }

    fn _walk_structure<Parent: Clone>(self: Rc<Self>, level: usize, incoming_edge: EdgeType, parent: Parent, visit: &Visitor<Parent>) {
        let parent = visit(self.clone(), level, incoming_edge, parent);
        let next_level = level + 1;
        match &*self {
            Envelope::Node { subject, assertions, .. } => {
                subject.clone()._walk_structure(next_level, EdgeType::Subject, parent.clone(), visit);
                for assertion in assertions {
                    assertion.clone()._walk_structure(next_level, EdgeType::Assertion, parent.clone(), visit);
                }
            },
            Envelope::Wrapped { envelope, .. } => {
                envelope.clone()._walk_structure(next_level, EdgeType::Wrapped, parent, visit);
            },
            Envelope::Assertion(assertion) => {
                assertion.predicate()._walk_structure(next_level, EdgeType::Predicate, parent.clone(), visit);
                assertion.object()._walk_structure(next_level, EdgeType::Object, parent, visit);
            },
            _ => {},
        }
    }

    fn walk_tree<Parent: Default + Clone>(self: Rc<Self>, visit: &Visitor<Parent>)
    {
        self._walk_tree(0, Default::default(), visit);
    }

    fn _walk_tree<Parent: Clone>(self: Rc<Self>, level: usize, parent: Parent, visit: &Visitor<Parent>) {
        let parent = visit(self.clone(), level, EdgeType::None, parent);
        let next_level = level + 1;
        match &*self {
            Envelope::Node { subject, assertions, .. } => {
                subject.clone()._walk_tree(next_level, parent.clone(), visit);
                for assertion in assertions {
                    assertion.clone()._walk_tree(next_level, parent.clone(), visit);
                }
            },
            Envelope::Wrapped { envelope, .. } => {
                envelope.clone()._walk_tree(next_level, parent, visit);
            },
            Envelope::Assertion(assertion) => {
                assertion.predicate()._walk_tree(next_level, parent.clone(), visit);
                assertion.object()._walk_tree(next_level, parent, visit);
            },
            _ => {},
        }
    }
}
