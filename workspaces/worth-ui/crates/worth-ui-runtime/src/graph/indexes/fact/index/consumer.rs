use crate::graph::{UiGraphAspectConsumerKind, UiGraphSnapshot};

use super::super::{UiGraphFactConsumerIdentity, UiGraphFactConsumerKey, UiGraphFactConsumerKind};

pub(super) fn consumer_key(
    snapshot: &UiGraphSnapshot,
    kind: UiGraphAspectConsumerKind,
) -> UiGraphFactConsumerKey {
    let (consumer_kind, node_identity) = match kind {
        UiGraphAspectConsumerKind::GraphNode(identity) => {
            (UiGraphFactConsumerKind::GraphNode, identity)
        }
        UiGraphAspectConsumerKind::MountEligibilitySlot(identity) => {
            let node_identity = snapshot
                .mount_eligibilities()
                .slot(identity)
                .expect("every indexed mount-eligibility consumer has a graph-owned slot")
                .graph_node_identity();
            (UiGraphFactConsumerKind::MountEligibilitySlot, node_identity)
        }
    };
    let node = snapshot
        .nodes()
        .iter()
        .find(|node| node.graph_node_identity() == node_identity)
        .expect("every indexed fact consumer has one graph node");
    let declaration = snapshot
        .core_indexes()
        .declaration_correspondence()
        .declaration_identity_for(node_identity)
        .expect("every indexed fact consumer has declaration correspondence");
    UiGraphFactConsumerKey::new(
        consumer_kind,
        declaration.authored_semantic_name(),
        node.repeated_instance_basis().identity_digest(),
    )
}

pub(super) fn consumer_identity(kind: UiGraphAspectConsumerKind) -> UiGraphFactConsumerIdentity {
    match kind {
        UiGraphAspectConsumerKind::GraphNode(identity) => {
            UiGraphFactConsumerIdentity::GraphNode(identity)
        }
        UiGraphAspectConsumerKind::MountEligibilitySlot(identity) => {
            UiGraphFactConsumerIdentity::MountEligibilitySlot(identity)
        }
    }
}
