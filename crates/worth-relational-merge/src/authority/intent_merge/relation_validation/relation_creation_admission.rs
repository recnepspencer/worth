use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::{
    CreatedEntityRef, EntityReference, RelationIdentity, RelationSpec,
};
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::endpoint_admission::validate_relation_endpoint_primitives;
use super::relation_identity_scan::existing_relation_targets_for_source;
use super::schema_relation_admission::require_registered_relation_kind;

pub(super) fn validate_relation_creation_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    default_cross_context_policy: crate::contracts::config::data::policies::CrossContextPolicy,
    candidates_scanned: &mut usize,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::contracts::identity::data::record_ids::RelationId>,
    spec: &RelationSpec,
) -> Result<(), CommitConflict> {
    validate_relation_creation(
        state,
        schema,
        default_cross_context_policy,
        candidates_scanned,
        created_entities,
        deleted_relations,
        spec.partition_id,
        spec.kind_id,
        &spec.source,
        &spec.target,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_relation_creation(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    default_cross_context_policy: crate::contracts::config::data::policies::CrossContextPolicy,
    candidates_scanned: &mut usize,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::contracts::identity::data::record_ids::RelationId>,
    partition_id: crate::contracts::identity::data::primitives::PartitionId,
    kind_id: crate::contracts::identity::data::primitives::KindId,
    source: &EntityReference,
    target: &EntityReference,
) -> Result<(), CommitConflict> {
    let relation_registration = require_registered_relation_kind(schema, kind_id)?;
    validate_relation_endpoint_primitives(
        state,
        relation_registration.cross_context_policy,
        default_cross_context_policy,
        source,
        target,
        created_entities,
    )?;
    reject_existing_relation_identity(
        state,
        candidates_scanned,
        partition_id,
        kind_id,
        source,
        &BTreeSet::from([target.clone()]),
        deleted_relations,
    )
}

pub(super) fn validate_bulk_relation_creation_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    default_cross_context_policy: crate::contracts::config::data::policies::CrossContextPolicy,
    candidates_scanned: &mut usize,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::contracts::identity::data::record_ids::RelationId>,
    partition_id: crate::contracts::identity::data::primitives::PartitionId,
    kind_id: crate::contracts::identity::data::primitives::KindId,
    endpoints: &[(EntityReference, EntityReference)],
) -> Result<(), CommitConflict> {
    let relation_registration = require_registered_relation_kind(schema, kind_id)?;
    let grouped_targets = validate_bulk_relation_endpoint_batch(
        state,
        relation_registration.cross_context_policy,
        default_cross_context_policy,
        created_entities,
        partition_id,
        kind_id,
        endpoints,
    )?;

    for (source, targets) in grouped_targets {
        reject_existing_relation_identity(
            state,
            candidates_scanned,
            partition_id,
            kind_id,
            &source,
            &targets,
            deleted_relations,
        )?;
    }
    Ok(())
}

fn validate_bulk_relation_endpoint_batch(
    state: &impl CurrentPartitionAccess,
    relation_cross_context_policy: crate::contracts::config::data::policies::CrossContextPolicy,
    default_cross_context_policy: crate::contracts::config::data::policies::CrossContextPolicy,
    created_entities: &BTreeSet<CreatedEntityRef>,
    partition_id: crate::contracts::identity::data::primitives::PartitionId,
    kind_id: crate::contracts::identity::data::primitives::KindId,
    endpoints: &[(EntityReference, EntityReference)],
) -> Result<BTreeMap<EntityReference, BTreeSet<EntityReference>>, CommitConflict> {
    let mut seen_batch_identities = BTreeSet::new();
    let mut targets_by_source = BTreeMap::new();
    for (source, target) in endpoints {
        reject_duplicate_relation_identity_in_batch(
            &mut seen_batch_identities,
            partition_id,
            kind_id,
            source,
            target,
        )?;
        validate_relation_endpoint_primitives(
            state,
            relation_cross_context_policy,
            default_cross_context_policy,
            source,
            target,
            created_entities,
        )?;
        targets_by_source
            .entry(source.clone())
            .or_insert_with(BTreeSet::new)
            .insert(target.clone());
    }
    Ok(targets_by_source)
}

fn reject_duplicate_relation_identity_in_batch(
    seen_batch_identities: &mut BTreeSet<RelationIdentity>,
    partition_id: crate::contracts::identity::data::primitives::PartitionId,
    kind_id: crate::contracts::identity::data::primitives::KindId,
    source: &EntityReference,
    target: &EntityReference,
) -> Result<(), CommitConflict> {
    let identity = RelationIdentity {
        partition_id,
        kind_id,
        source: source.clone(),
        target: target.clone(),
    };
    if seen_batch_identities.insert(identity) {
        return Ok(());
    }

    Err(CommitConflict::new(
        ConflictClass::DuplicateRelationIdentity {
            detail: "duplicate relation identity within bulk batch".to_string(),
        },
    ))
}

fn reject_existing_relation_identity(
    state: &impl CurrentPartitionAccess,
    candidates_scanned: &mut usize,
    partition_id: crate::contracts::identity::data::primitives::PartitionId,
    kind_id: crate::contracts::identity::data::primitives::KindId,
    source: &EntityReference,
    targets: &BTreeSet<EntityReference>,
    deleted_relations: &BTreeSet<crate::contracts::identity::data::record_ids::RelationId>,
) -> Result<(), CommitConflict> {
    if existing_relation_targets_for_source(
        state,
        candidates_scanned,
        partition_id,
        kind_id,
        source,
        targets,
        deleted_relations,
    ) {
        return Err(CommitConflict::new(
            ConflictClass::DuplicateRelationIdentity {
                detail: "duplicate relation identity".to_string(),
            },
        ));
    }
    Ok(())
}
