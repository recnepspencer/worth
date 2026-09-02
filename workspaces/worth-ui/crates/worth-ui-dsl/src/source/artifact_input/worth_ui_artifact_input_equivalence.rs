use crate::source::{WorthUiArtifactInput, WorthUiArtifactInputNode};

pub(crate) struct WorthUiArtifactInputEquivalentShape;

impl WorthUiArtifactInputEquivalentShape {
    pub(crate) fn packages_are_equivalent(
        left: &WorthUiArtifactInput,
        right: &WorthUiArtifactInput,
    ) -> bool {
        left.module_ids() == right.module_ids()
            && left.module_ids().iter().all(|module_id| {
                left.module(module_id)
                    .zip(right.module(module_id))
                    .is_some_and(|(left_module, right_module)| {
                        left_module.nodes().len() == right_module.nodes().len()
                            && left_module
                                .nodes()
                                .iter()
                                .zip(right_module.nodes().iter())
                                .all(|(left_node, right_node)| {
                                    nodes_are_equivalent(left_node, right_node)
                                })
                    })
            })
    }
}

fn nodes_are_equivalent(left: &WorthUiArtifactInputNode, right: &WorthUiArtifactInputNode) -> bool {
    match (left, right) {
        (WorthUiArtifactInputNode::Import(left), WorthUiArtifactInputNode::Import(right)) => {
            left.target() == right.target()
        }
        (WorthUiArtifactInputNode::Component(left), WorthUiArtifactInputNode::Component(right))
        | (WorthUiArtifactInputNode::Surface(left), WorthUiArtifactInputNode::Surface(right))
        | (WorthUiArtifactInputNode::Binding(left), WorthUiArtifactInputNode::Binding(right)) => {
            left.name_text() == right.name_text()
                && left.body_atoms() == right.body_atoms()
                && left.appearance_role_attachment() == right.appearance_role_attachment()
        }
        (
            WorthUiArtifactInputNode::QueryScalar(left),
            WorthUiArtifactInputNode::QueryScalar(right),
        )
        | (
            WorthUiArtifactInputNode::QueryCollection(left),
            WorthUiArtifactInputNode::QueryCollection(right),
        ) => left.name_text() == right.name_text() && left.body_atoms() == right.body_atoms(),
        (WorthUiArtifactInputNode::Token(left), WorthUiArtifactInputNode::Token(right)) => {
            left.name_text() == right.name_text() && left.value_text() == right.value_text()
        }
        (
            WorthUiArtifactInputNode::SemanticArtifact(left),
            WorthUiArtifactInputNode::SemanticArtifact(right),
        ) => left.declaration() == right.declaration(),
        (
            WorthUiArtifactInputNode::AppearanceRole(left),
            WorthUiArtifactInputNode::AppearanceRole(right),
        ) => left.role() == right.role(),
        (WorthUiArtifactInputNode::Backdrop(left), WorthUiArtifactInputNode::Backdrop(right)) => {
            left.declaration() == right.declaration()
        }
        _ => false,
    }
}
