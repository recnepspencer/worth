use std::collections::BTreeSet;

use crate::capabilities::{SchemaSource, StorageRead};
use crate::runtime::RuntimeInstrumentation;
use crate::transactions::data::{
    CommitConflict, ConflictClass, CreatedEntityRef, UpdateRelationEndpointsIntent,
};

use super::endpoint_admission::validate_relation_endpoint_primitives;
use super::relation_identity_scan::existing_relation_targets_for_source;
use super::relation_target_admission::validate_existing_relation_target;
use super::schema_relation_admission::require_registered_relation_kind;

pub(super) fn validate_relation_endpoint_update_intent(
    state: &impl StorageRead,
    schema_source: &impl SchemaSource,
    default_cross_context_policy: crate::config::data::CrossContextPolicy,
    instrumentation: &RuntimeInstrumentation,
    created_entities: &BTreeSet<CreatedEntityRef>,
    spec: &UpdateRelationEndpointsIntent,
) -> Result<(), CommitConflict> {
    validate_existing_relation_target(state, spec.relation_id)?;
    let relation_registration = require_registered_relation_kind(schema_source, spec.kind_id)?;
    validate_relation_endpoint_primitives(
        state,
        relation_registration.cross_context_policy,
        default_cross_context_policy,
        &spec.source,
        &spec.target,
        created_entities,
    )?;
    reject_endpoint_update_duplicate_identity(state, instrumentation, spec)
}

fn reject_endpoint_update_duplicate_identity(
    state: &impl StorageRead,
    instrumentation: &RuntimeInstrumentation,
    spec: &UpdateRelationEndpointsIntent,
) -> Result<(), CommitConflict> {
    if existing_relation_targets_for_source(
        state,
        instrumentation,
        spec.relation_id.partition_id,
        spec.kind_id,
        &spec.source,
        &BTreeSet::from([spec.target.clone()]),
        &BTreeSet::from([spec.relation_id]),
    ) {
        return Err(CommitConflict::new(
            ConflictClass::DuplicateRelationIdentity {
                detail: "duplicate relation identity".to_string(),
            },
        ));
    }
    Ok(())
}
