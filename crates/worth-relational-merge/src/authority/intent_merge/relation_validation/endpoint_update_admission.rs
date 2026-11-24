use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use std::collections::BTreeSet;

use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::intents::mutation_intent::UpdateRelationEndpointsIntent;
use crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use crate::contracts::transactions::data::primitives::CreatedEntityRef;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

use super::endpoint_admission::validate_relation_endpoint_primitives;
use super::relation_identity_scan::existing_relation_targets_for_source;
use super::relation_target_admission::validate_existing_relation_target;
use super::schema_relation_admission::require_registered_relation_kind;

pub(super) fn validate_relation_endpoint_update_intent(
    state: &impl CurrentPartitionAccess,
    schema: &RelationalSchemaRegistry,
    default_cross_context_policy: crate::contracts::config::data::policies::CrossContextPolicy,
    candidates_scanned: &mut usize,
    created_entities: &BTreeSet<CreatedEntityRef>,
    spec: &UpdateRelationEndpointsIntent,
) -> Result<(), CommitConflict> {
    validate_existing_relation_target(state, spec.relation_id)?;
    let relation_registration = require_registered_relation_kind(schema, spec.kind_id)?;
    validate_relation_endpoint_primitives(
        state,
        relation_registration.cross_context_policy,
        default_cross_context_policy,
        &spec.source,
        &spec.target,
        created_entities,
    )?;
    reject_endpoint_update_duplicate_identity(state, candidates_scanned, spec)
}

fn reject_endpoint_update_duplicate_identity(
    state: &impl CurrentPartitionAccess,
    candidates_scanned: &mut usize,
    spec: &UpdateRelationEndpointsIntent,
) -> Result<(), CommitConflict> {
    if existing_relation_targets_for_source(
        state,
        candidates_scanned,
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
