use std::collections::BTreeMap;

use crate::runtime::{
    WorthUiIdentityMatchCounters, WorthUiIdentityMatchDenial, WorthUiIdentityMatchNode,
    WorthUiIdentityMatchNodeSide, WorthUiRepeatedTemplateIdentity,
};
use crate::source::{WorthUiArtifact, WorthUiArtifactIdentitySeed, WorthUiArtifactNode};

use super::denial_assembly::duplicate_identity_denial;
use super::structure_digests::{
    durable_state_is_eligible, node_durable_state_eligibility, node_identity_seed,
};
use super::types::IndexedIdentityNode;
use crate::runtime::replacement::artifact_durable_state_definition::{
    node_has_restorable_splitter_state, node_resize_contract_id, node_resize_permission,
    node_resize_shape_digest,
};

pub(super) fn index_artifact_nodes(
    artifact: &WorthUiArtifact,
    side: WorthUiIdentityMatchNodeSide,
    counters: &mut WorthUiIdentityMatchCounters,
) -> Result<BTreeMap<String, IndexedIdentityNode>, WorthUiIdentityMatchDenial> {
    let mut index = BTreeMap::new();
    for module_id in artifact.module_ids() {
        let Some(module) = artifact.module(module_id) else {
            continue;
        };
        for node in module.nodes() {
            match side {
                WorthUiIdentityMatchNodeSide::Active => counters.record_active_node_indexed(),
                WorthUiIdentityMatchNodeSide::Candidate => counters.record_candidate_node_indexed(),
            }
            let identity_seed = node_identity_seed(node);
            let identity_basis = canonical_match_identity_basis(node, identity_seed);
            counters.record_stable_seed_lookup();
            if !identity_seed.is_stable() {
                continue;
            }
            reject_position_only_repeated_template_identity(node, identity_seed, side, *counters)?;
            let match_node =
                WorthUiIdentityMatchNode::new(super::super::WorthUiIdentityMatchNodeInput {
                    side,
                    handle: node.handle().clone(),
                    identity_basis: identity_basis.to_owned(),
                    authored_provenance_digest: node.authored_provenance_digest(),
                    semantic_meaning: node.clone(),
                    stable_identity: identity_seed.is_stable(),
                    durable_state_eligible: durable_state_is_eligible(
                        node_durable_state_eligibility(node),
                    ),
                    has_restorable_splitter_state: node_has_restorable_splitter_state(node),
                    resize_contract_id: node_resize_contract_id(node),
                    resize_permission: node_resize_permission(node),
                    resize_shape_digest: node_resize_shape_digest(node),
                });
            insert_indexed_identity_node(
                &mut index,
                side,
                identity_basis,
                identity_basis.to_owned(),
                IndexedIdentityNode {
                    node: match_node.clone(),
                },
                counters,
            )?;
        }
    }
    Ok(index)
}

fn canonical_match_identity_basis<'node>(
    node: &'node WorthUiArtifactNode,
    seed: &'node WorthUiArtifactIdentitySeed,
) -> &'node str {
    match node {
        WorthUiArtifactNode::Binding(binding) => binding
            .view_binding_reference()
            .view_binding()
            .id()
            .as_str(),
        WorthUiArtifactNode::Import(_)
        | WorthUiArtifactNode::Component(_)
        | WorthUiArtifactNode::Surface(_)
        | WorthUiArtifactNode::Token(_) => seed.basis(),
    }
}

fn reject_position_only_repeated_template_identity(
    node: &WorthUiArtifactNode,
    identity_seed: &WorthUiArtifactIdentitySeed,
    side: WorthUiIdentityMatchNodeSide,
    mut counters: WorthUiIdentityMatchCounters,
) -> Result<(), WorthUiIdentityMatchDenial> {
    if WorthUiRepeatedTemplateIdentity::is_position_only(identity_seed.basis()) {
        match side {
            WorthUiIdentityMatchNodeSide::Active => counters.record_duplicate_active_identity(),
            WorthUiIdentityMatchNodeSide::Candidate => {
                counters.record_duplicate_candidate_identity();
            }
        }
        Err(
            WorthUiIdentityMatchDenial::PositionOnlyRepeatedTemplateIdentity {
                identity_basis: identity_seed.basis().to_owned(),
                node_summary: format!(
                    "{:?}:{}:{}",
                    node.handle().kind(),
                    node.handle().module_id().as_str(),
                    node.handle().node_index()
                ),
                counters: Box::new(counters),
            },
        )
    } else {
        Ok(())
    }
}

fn insert_indexed_identity_node(
    index: &mut BTreeMap<String, IndexedIdentityNode>,
    side: WorthUiIdentityMatchNodeSide,
    identity_basis: &str,
    owned_identity_basis: String,
    indexed_node: IndexedIdentityNode,
    counters: &mut WorthUiIdentityMatchCounters,
) -> Result<(), WorthUiIdentityMatchDenial> {
    if let Some(previous) = index.get(identity_basis) {
        classify_same_side_identity_kind_alignment(
            side,
            identity_basis,
            previous,
            &indexed_node.node,
            counters,
        )?;
        return duplicate_identity_denial(
            side,
            owned_identity_basis,
            previous.node.node_summary(),
            indexed_node.node.node_summary(),
            counters,
        );
    }

    index.insert(owned_identity_basis, indexed_node);
    Ok(())
}

/// Named classifier: same-side stable seeds must share identity kind.
fn classify_same_side_identity_kind_alignment(
    side: WorthUiIdentityMatchNodeSide,
    identity_basis: &str,
    previous: &IndexedIdentityNode,
    next: &WorthUiIdentityMatchNode,
    counters: &mut WorthUiIdentityMatchCounters,
) -> Result<(), WorthUiIdentityMatchDenial> {
    if previous.node.kind() == next.kind() {
        return Ok(());
    }

    counters.record_identity_kind_mismatch();
    match side {
        WorthUiIdentityMatchNodeSide::Active => {
            Err(WorthUiIdentityMatchDenial::ActiveIdentityKindMismatch {
                identity_basis: identity_basis.to_owned(),
                first_kind: previous.node.kind(),
                second_kind: next.kind(),
                first_node_summary: previous.node.node_summary(),
                second_node_summary: next.node_summary(),
                counters: Box::new(*counters),
            })
        }
        WorthUiIdentityMatchNodeSide::Candidate => {
            Err(WorthUiIdentityMatchDenial::CandidateIdentityKindMismatch {
                identity_basis: identity_basis.to_owned(),
                first_kind: previous.node.kind(),
                second_kind: next.kind(),
                first_node_summary: previous.node.node_summary(),
                second_node_summary: next.node_summary(),
                counters: Box::new(*counters),
            })
        }
    }
}
