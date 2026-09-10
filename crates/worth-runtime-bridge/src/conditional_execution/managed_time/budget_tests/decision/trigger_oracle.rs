use super::super::layout_oracle as layout;
use crate::correspondence::{
    BridgeDeliveredCorrespondenceChange, BridgeDeliveredCorrespondenceChangeSet,
    BridgeSemanticLocality,
};
use std::{collections::HashMap, mem::size_of, sync::Arc};
use worth_foundational::facade::{AspectShape, CanonicalFieldPath, FieldDeclaration, FieldKey};

pub(super) fn expected(set: &BridgeDeliveredCorrespondenceChangeSet) -> u64 {
    // The decision core includes the inline trigger and its guard.
    let mut bytes = 0;
    let basis = set.basis();
    let dependency = set.dependency();
    // Independent oracle: collect actual allocations in the finite source tuple.
    // Equal text in a different allocation must occupy a separate map entry.
    let source_allocations: HashMap<*const (), usize> = [
        &basis.source_installation_identity,
        &basis.source_basis,
        &basis.source_authority_binding_identity,
        &basis.declared_graph_role,
        &basis.graph_participation_identity,
        &basis.graph_adapter_identity,
        &dependency.source_installation_identity,
        &dependency.source_basis,
        &dependency.source_authority_binding_identity,
        &dependency.declared_graph_role,
        &dependency.graph_participation_identity,
        &dependency.graph_adapter_identity,
    ]
    .into_iter()
    .map(|text| (Arc::as_ptr(text).cast::<()>(), text.len()))
    .collect();
    bytes += source_allocations
        .values()
        .map(|length| layout::text(*length))
        .sum::<u64>();
    if let Some(profile) = &basis.authoritative_source_profile {
        bytes += layout::text(profile.adapter_semantic_identity().len());
    }
    bytes += size_of::<worth_signal::facade::PartitionToken>() as u64
        * basis.signal_partitions.len() as u64;
    for partition in &basis.signal_partitions {
        bytes += partition.0.len() as u64;
    }
    bytes += layout::text(dependency.source_node_identity.len());
    if let Some(stage) = &dependency.source_stage_identity {
        bytes += layout::text(stage.len());
    }
    if let BridgeSemanticLocality::SourcePartition(partition) = &dependency.locality {
        bytes += partition.as_str().len() as u64;
    }
    bytes += dependency.contract.key().as_str().len() as u64;
    if let AspectShape::Struct(shape) = dependency.contract.shape() {
        bytes += size_of::<FieldDeclaration>() as u64 * shape.fields().len() as u64;
        for field in shape.fields() {
            bytes += field.key().as_str().len() as u64;
        }
    }
    bytes +=
        size_of::<CanonicalFieldPath>() as u64 * dependency.projection_mask.paths().len() as u64;
    for item in dependency.projection_mask.paths() {
        bytes += path(item);
    }
    bytes += binding(&dependency.binding);
    bytes += size_of::<worth_foundational::facade::AuthoritativeAspectChangeKind>() as u64
        * dependency.relevant_changes.len() as u64;
    bytes += identity(set.commit_identity())
        + identity(set.patch_identity())
        + identity(set.snapshot_identity())
        + identity(set.branch_identity());
    bytes += size_of::<BridgeDeliveredCorrespondenceChange>() as u64 * set.changes().len() as u64;
    assert!(
        !set.changes().is_empty(),
        "real owner delivered actual changes"
    );
    for change in set.changes() {
        if let Some(entity) = change.entity_identity() {
            bytes += layout::text(entity.len());
        }
        if let Some(change) = change.semantic_change() {
            bytes += change.aspect_key().as_str().len() as u64 + binding(change.binding());
            if let Some(item) = change.field_path() {
                bytes += path(item);
            }
        }
    }
    bytes
}

fn path(path: &CanonicalFieldPath) -> u64 {
    (size_of::<FieldKey>() * path.fields().len()
        + path
            .fields()
            .iter()
            .map(|field| field.as_str().len())
            .sum::<usize>()) as u64
}

fn binding(binding: &worth_foundational::facade::AspectBinding) -> u64 {
    match binding {
        worth_foundational::facade::AspectBinding::EntityField { field }
        | worth_foundational::facade::AspectBinding::RelationField { field } => {
            field.as_str().len() as u64
        }
        _ => 0,
    }
}

fn identity<Tag>(identity: &crate::identity::BridgeIdentity<Tag>) -> u64 {
    layout::text(identity.as_str().len())
        + match identity.payload() {
            crate::identity::BridgeIdentityPayload::RelationalBranch { branch_id } => {
                layout::text(branch_id.len())
            }
            _ => 0,
        }
}
