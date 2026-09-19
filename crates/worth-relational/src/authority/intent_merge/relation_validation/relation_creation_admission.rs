use std::collections::{BTreeMap, BTreeSet};

use crate::capabilities::{SchemaSource, StorageRead};
use crate::runtime::RuntimeInstrumentation;
use crate::transactions::data::{
    CommitConflict, ConflictClass, CreatedEntityRef, EntityReference, RelationIdentity,
    RelationSpec,
};

use super::endpoint_admission::validate_relation_endpoint_primitives;
use super::relation_identity_scan::existing_relation_targets_for_source;
use super::schema_relation_admission::require_registered_relation_kind;

pub(super) fn validate_relation_creation_intent(
    state: &impl StorageRead,
    schema_source: &impl SchemaSource,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    instrumentation: &RuntimeInstrumentation,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::identity::data::RelationId>,
    spec: &RelationSpec,
) -> Result<(), CommitConflict> {
    validate_relation_creation(
        state,
        schema_source,
        default_cross_context_policy,
        instrumentation,
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
    state: &impl StorageRead,
    schema_source: &impl SchemaSource,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    instrumentation: &RuntimeInstrumentation,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::identity::data::RelationId>,
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
    source: &EntityReference,
    target: &EntityReference,
) -> Result<(), CommitConflict> {
    let relation_registration = require_registered_relation_kind(schema_source, kind_id)?;
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
        instrumentation,
        partition_id,
        kind_id,
        source,
        &BTreeSet::from([target.clone()]),
        deleted_relations,
    )
}

pub(super) fn validate_bulk_relation_creation_intent(
    state: &impl StorageRead,
    schema_source: &impl SchemaSource,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    instrumentation: &RuntimeInstrumentation,
    created_entities: &BTreeSet<CreatedEntityRef>,
    deleted_relations: &BTreeSet<crate::identity::data::RelationId>,
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
    endpoints: &[(EntityReference, EntityReference)],
) -> Result<(), CommitConflict> {
    let relation_registration = require_registered_relation_kind(schema_source, kind_id)?;
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
            instrumentation,
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
    state: &impl StorageRead,
    relation_cross_context_policy: crate::config::data::CrossContextPolicy,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    created_entities: &BTreeSet<CreatedEntityRef>,
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
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
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
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
    state: &impl StorageRead,
    instrumentation: &RuntimeInstrumentation,
    partition_id: crate::identity::data::PartitionId,
    kind_id: crate::identity::data::KindId,
    source: &EntityReference,
    targets: &BTreeSet<EntityReference>,
    deleted_relations: &BTreeSet<crate::identity::data::RelationId>,
) -> Result<(), CommitConflict> {
    if existing_relation_targets_for_source(
        state,
        instrumentation,
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
