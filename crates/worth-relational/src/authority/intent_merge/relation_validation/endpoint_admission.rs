use std::collections::BTreeSet;

use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::{CommitConflict, ConflictClass, CreatedEntityRef, EntityReference};

use super::super::record_lookup::entity_exists_in_state;

pub(super) fn validate_relation_endpoint_primitives(
    state: &impl CurrentPartitionAccess,
    relation_cross_context_policy: crate::config::data::CrossContextPolicy,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    source: &EntityReference,
    target: &EntityReference,
    created_entities: &BTreeSet<CreatedEntityRef>,
) -> Result<(), CommitConflict> {
    validate_cross_context_endpoint_policy(
        relation_cross_context_policy,
        default_cross_context_policy,
        source,
        target,
    )?;
    validate_endpoint_entities_are_live_in_commit_scope(state, source, target, created_entities)
}

fn validate_cross_context_endpoint_policy(
    relation_cross_context_policy: crate::config::data::CrossContextPolicy,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    source: &EntityReference,
    target: &EntityReference,
) -> Result<(), CommitConflict> {
    if source.partition_id() == target.partition_id() {
        return Ok(());
    }

    let effective_cross_context_policy = match relation_cross_context_policy {
        crate::config::data::CrossContextPolicy::SchemaControlled => default_cross_context_policy,
        explicit_policy => explicit_policy,
    };
    if effective_cross_context_policy == crate::config::data::CrossContextPolicy::AllowExplicit {
        return Ok(());
    }

    Err(CommitConflict::new(
        ConflictClass::InvalidRelationEndpoint {
            detail: "cross-context relation endpoints are not allowed for this relation kind"
                .to_string(),
        },
    ))
}

fn validate_endpoint_entities_are_live_in_commit_scope(
    state: &impl CurrentPartitionAccess,
    source: &EntityReference,
    target: &EntityReference,
    created_entities: &BTreeSet<CreatedEntityRef>,
) -> Result<(), CommitConflict> {
    if entity_reference_exists_in_commit_scope(state, source, created_entities)
        && entity_reference_exists_in_commit_scope(state, target, created_entities)
    {
        return Ok(());
    }

    Err(CommitConflict::new(
        ConflictClass::InvalidRelationEndpoint {
            detail: "relation endpoints must be live entities".to_string(),
        },
    ))
}

fn entity_reference_exists_in_commit_scope(
    state: &impl CurrentPartitionAccess,
    entity_reference: &EntityReference,
    created_entities: &BTreeSet<CreatedEntityRef>,
) -> bool {
    match entity_reference {
        EntityReference::Existing(entity_id) => entity_exists_in_state(state, *entity_id),
        EntityReference::Created(created) => created_entities.contains(created),
    }
}
