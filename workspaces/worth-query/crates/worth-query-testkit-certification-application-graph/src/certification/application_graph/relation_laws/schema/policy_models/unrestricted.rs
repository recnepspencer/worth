//! One relation law model.
use super::super::*;
pub struct UnrestrictedRecords;
impl Fragment for UnrestrictedRecords {
    const NAME: &'static str = "graph_records";
    type Bindings = GraphBindings;
    type Members = (ExternalMapping, Principal, Leaf, Node);
}

impl GraphModel for UnrestrictedRecords {
    type Node = Node;
    type Precedes = Precedes;
    type Feeds = Feeds;
    type Key = FieldSelection<Self, Node, NodeFacts, u64>;
    type Weight = RelationFieldSelection<Self, Node, Precedes, EdgeFacts, u64>;
    fn node_key() -> Self::Key {
        Self::Key::at("NodeKey")
    }
    fn edge_weight() -> Option<Self::Weight> {
        None
    }
}
