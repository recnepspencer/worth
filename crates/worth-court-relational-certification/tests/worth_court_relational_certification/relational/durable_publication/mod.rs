//! Public Native durability workflows, independent of source-unit privacy.
mod aggregate_validation;
mod associated_effects;
mod checkpoint_capture_custody;
mod checkpoint_field_revisions;
mod checkpoint_transition_compaction;
mod checkpoint_transition_gap_replay;
mod checkpoint_truth_observation;
mod cold_envelope_custody;
mod home_readmission_metadata;
mod index_catalog;
mod initial_schema_recovery;
mod lineage_frontiers;
mod native_checkpoint_capture_owners;
mod prepared_home_schema;
mod recorded_basis;
mod recorded_component_checkpoint;
mod recorded_component_custody;
mod recorded_component_property;
mod recorded_component_recovery;
mod recorded_component_validation;
mod recovered_associated_effects;
mod recovered_publication;
mod result_snapshot_cleanup;
mod semantic_result;
pub(crate) mod source_initial_inputs;
mod world_selection_disposed_source;

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
pub(in crate::relational) mod branch_history_publications;
mod branch_history_recovery;
mod definition_lookup_snapshot;
mod entity_projection_history;
mod field_revision_history;

mod commit_meaning;
mod original_input_observation;

mod checkpoint_recovery_observation;
mod native_checkpoint_reopen;

mod checkpoint_branch_targets;

mod checkpoint_branch_lifecycle;
mod concurrent_branch_frontier;
mod historical_fork_lifecycle;

mod checkpoint_tail_and_indexes;

mod checkpoint_successor_authority;

mod checkpoint_source_capture;

mod association_checkpoint_journeys;

mod medium_publication_journeys;

mod medium_member_retention;

mod caller_association_journeys;

mod recorded_original_checkpoint_journeys;

mod checkpoint_protected_history;

mod checkpoint_content_root;

mod checkpoint_transition_continuation;

mod medium_recovered_continuation;

mod checkpoint_sparse_lineage;

mod checkpoint_descriptor_continuity;

mod checkpoint_schema_versions;

mod checkpoint_declared_schema;

mod checkpoint_inspection_observation;

mod checkpoint_inspection_journeys;

mod checkpoint_record_replacement;

mod checkpoint_transition_descriptors;

mod checkpoint_schema_plan_refusals;

mod checkpoint_canonical_neutrality;

mod checkpoint_managed_text;

mod canonical_publication_touches;
mod checkpoint_abandoned_keys;
mod checkpoint_bulk_bindings;
mod checkpoint_custom_versions;
mod checkpoint_empty_head;
mod checkpoint_index_affinity;
mod checkpoint_index_generations;
mod checkpoint_index_lagging;
mod checkpoint_index_recovery;
mod checkpoint_lineage_authority;
mod checkpoint_lineage_issuance;
mod checkpoint_lineage_publication;

mod checkpoint_catalog_originals;

mod checkpoint_pinned_root;

mod checkpoint_recovered_readers;

mod checkpoint_root_publications;

mod checkpoint_shared_partition;

mod checkpoint_birth_refusal;

mod checkpoint_generation_identity;

mod checkpoint_topology_reidentification;

mod checkpoint_inspection_self_edge;
mod checkpoint_shipping_lookup;
mod checkpoint_truth_equivalence;

mod checkpoint_subscriber_source;

mod native_checkpoint_partitions;
