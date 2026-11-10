//! Public Native durability workflows, independent of source-unit privacy.
mod associated_effects;
mod cold_envelope_custody;
mod index_catalog;
mod initial_schema_recovery;
mod recorded_basis;
mod recorded_component_checkpoint;
mod recorded_component_recovery;
mod recorded_component_validation;
mod recovered_associated_effects;
mod recovered_publication;
mod result_snapshot_cleanup;
mod semantic_result;
pub(crate) mod source_initial_inputs;

use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::{
        RecordedRelationalComponentReference, RecordedRelationalComponentReference as Recorded,
        RecordedRelationalComponentTarget, RecordedRelationalComponentTarget as Target,
        RelationalRecordedComponentRefusal,
    },
    durability::{
        RecoveryVerificationMode, RelationalAssociatedEffectRefusal as Refusal,
        RelationalAssociatedEffectSettlement as Settlement,
        RelationalCanonicalCommitArtifact as Artifact, RelationalRecordedComponentAdmissionCause,
    },
    identity::{KindId, PartitionId},
    mvcc::{RelationalCallerAssociation as Association, RelationalPublicationOutcome},
    transactions::{
        CreateIntent, CreatedEntityRef, EntityReference, MutationIntent, RecordRef,
        RelationalCreatedEndpoint as Endpoint, WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_medium::TestDurableMedium,
    durable_publication::{
        assert_recorded_basis, batch_create, capture, create_entity, create_entity_outcome,
        current, durable_runtime, field_patch, memory_runtime, original,
        original_with_retained_successor, perform, recover, source_journey,
    },
};

mod encoded_checkpoint_backing;
mod partition_frontier;
mod validation_posture;

mod branch_aspect_history;
mod branch_checkpoint_journeys;
mod branch_endpoint_history;
mod branch_history_publications;
mod branch_history_recovery;
mod definition_lookup_snapshot;

mod merge_history_planning;
mod merge_policy_evidence;
mod merge_visibility_evidence;

mod merge_basis_journeys;
mod merge_packet_canonical_journey;
mod merge_packet_journeys;
mod merge_proof_journey;
mod merge_request_journeys;
mod merge_request_lowering_journey;

mod merge_collaboration_witness_journeys;
mod merge_correspondence_journeys;
mod merge_schema_witness_journeys;
mod merge_strategy_witness_journey;
mod merge_support_witness_journey;
mod merge_witness_authoring;
mod merge_witness_schema;

mod commit_meaning;
mod original_input_observation;

mod merge_branch_deletion;
mod merge_counter_recovery;
mod merge_declared_policy_boundaries;
mod merge_deleted_recovery;
mod merge_deletion_block_reasons;
mod merge_deletion_denials;
mod merge_descriptive_touches;
mod merge_envelope_publication;
mod merge_execution_authoring;
mod merge_execution_certification;
mod merge_execution_certification_cases;
mod merge_execution_diagnostics;
mod merge_execution_observation;
mod merge_execution_recovery;
mod merge_execution_schema;
mod merge_failure_diagnostics;
mod merge_feature_execution;
mod merge_history_shape;
mod merge_inspection_rows;
mod merge_last_writer_wins;
mod merge_local_topology;
mod merge_planning_recovery;
mod merge_prepared_verification;
mod merge_readiness_journeys;
mod merge_recovery_diagnostics;
mod merge_region_topology;
mod merge_source_admission;
mod merge_stale_execution;
mod merge_unknown_source_admission;
