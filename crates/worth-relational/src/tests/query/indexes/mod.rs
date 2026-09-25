mod bounded_entity_field_lookup;
mod bounded_relation_join_lookup;
mod branch_scope;
mod candidate_index_publication;
mod canonical_build_basis;
mod entity_field_lookup;
mod exact_observation;
mod generation_identity_recovery;
mod generation_routing;
mod generation_selection_locality;
mod historical_relation_field_lookup;
mod main_branch_unique_recovery;
mod maintenance;
mod maintenance_lifecycle;
mod maintenance_locality;
mod maintenance_slot_reuse;
mod parity_observability;
mod recovery_and_execution_models;
mod related_entity_ordered_lookup;
mod related_entity_schema_recovery;
mod relation_field_lookup;

use crate::facade::history::BranchId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::query::{
    IndexParityMode, IndexQueryRejectionClass, QueryAccessContract, QueryAccessPath,
};
use crate::facade::runtime::RelationalExecutionModel;
use crate::facade::transactions::RecordRef;
use crate::tests::support::*;
use std::sync::Arc;

fn runtime_with_index_field_aspects() -> RelationalRuntime {
    runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
}

fn persisted_runtime_with_index_field_aspects() -> RelationalRuntime {
    persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
}

fn sampled_plan_key_for(
    generation_id: crate::facade::indexes::DerivedIndexGenerationId,
    version_id: crate::facade::identity::VersionId,
    should_sample: bool,
) -> crate::facade::query::DeterministicQueryPlanKey {
    for key in 1u128..512 {
        let sample_key = key ^ ((generation_id.0 as u128) << 64) ^ (version_id.0 as u128);
        let sampled = sample_key.is_multiple_of(8);
        if sampled == should_sample {
            return crate::facade::query::DeterministicQueryPlanKey(key);
        }
    }
    panic!("unable to derive deterministic sampled plan key");
}

// CONTRACT: derived_index
// LANES: success, access_contract, determinism
