//! Serialize the actual immutable route selector without collecting or resetting it.
use super::{PositionNode, RequiredOutputPublicationPositions, RequiredOutputRoutePosition};
use serde::{ser::SerializeStruct, Serialize, Serializer};

/// Borrowed DATA emitted only while the owning checkpoint capture holds its root.
pub(crate) struct CheckpointOutputPositions<'a>(&'a RequiredOutputPublicationPositions);

impl RequiredOutputPublicationPositions {
    pub(crate) fn checkpoint_view(&self) -> CheckpointOutputPositions<'_> {
        CheckpointOutputPositions(self)
    }
}

impl Serialize for CheckpointOutputPositions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_struct("CheckpointOutputPositions", 2)?;
        record.serialize_field("unresolved_prefix", &self.0.unresolved_prefix)?;
        record.serialize_field("root", &Node(self.0.root.as_deref()))?;
        record.end()
    }
}

struct Node<'a>(Option<&'a PositionNode>);

impl Serialize for Node<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Some(node) = self.0 else {
            return serializer.serialize_none();
        };
        let publication = match node.publication {
            RequiredOutputRoutePosition::Published(commit) => (0_u8, commit),
            RequiredOutputRoutePosition::CodecRetired(commit) => (1_u8, commit),
        };
        // The existing persistent AVL bounds recursion. Its actual shape is
        // carried, so sizing and emission need neither a traversal stack nor a
        // preliminary all-node count outside the request's serializer sink.
        let mut record = serializer.serialize_struct("CheckpointOutputPositionNode", 6)?;
        record.serialize_field("family", &node.route.family)?;
        record.serialize_field("source", &node.route.source)?;
        record.serialize_field("partition", &node.route.partition)?;
        record.serialize_field("publication", &publication)?;
        record.serialize_field("left", &Node(node.left.as_deref()))?;
        record.serialize_field("right", &Node(node.right.as_deref()))?;
        record.end()
    }
}
