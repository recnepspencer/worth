use super::fixtures::schema_transition_for_subscriber_impact;
use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::publication::SubscriberResumeRequest;
use crate::facade::schema::{SchemaReconciliationPolicy, SchemaSubscriberImpact, SchemaVersionId};
use crate::replay::data::{
    digest_diagnostics_batch_surface, digest_patch_batch_surface,
    digest_subscriber_boundary_cdc_surface, digest_subscriber_continuation_counter_pair,
};
use crate::tests::support::*;

#[test]
fn cdc_certification_schema_boundary_continuation_is_explained_and_counted() {
    let mut runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
    let _ = create_entity_outcome(&runtime, "anchor");

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut txn = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn.push_batch(
        batch_create("after-boundary"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    txn.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();

    runtime.performance_access().reset_counters();
    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(16))
        .unwrap();
    let counters = runtime.performance_access().counters();

    assert_eq!(
        batch.continuation.continuation_outcome(),
        crate::facade::schema::SchemaContinuationClassification::ContinueWithVisibleBridge
    );
    assert!(batch
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::SubscriberContractEvaluated));
    assert!(batch
        .diagnostics
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::SubscriberBoundaryEvaluated));
    assert_eq!(counters.subscriber_resume_evaluations, 1);
    assert_eq!(counters.subscriber_continue_visible_bridge_count, 1);
    assert_eq!(counters.schema_normalized_descriptor_compositions, 1);
}

#[test]
fn diff_cdc_truth_parity_test() {
    let mut runtime = persisted_runtime_with_test_schema();
    let baseline = create_entity_outcome(&runtime, "anchor");
    let baseline_checkpoint =
        checkpoint_for_schema_version(baseline.patch_position(), SchemaVersionId(1));

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut txn_v2 = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(2),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn_v2
        .push_batch(
            batch_create("after-v2"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    txn_v2
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(3),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let mut txn_v3 = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime)
                .with_schema_transition(
                    schema_transition_for_subscriber_impact(
                        SchemaVersionId(3),
                        SchemaSubscriberImpact::ConsumableSurfaceChanged,
                    ),
                    Some(SchemaReconciliationPolicy::PreserveInformation),
                );
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn_v3
        .push_batch(
            batch_create("after-v3"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    txn_v3
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    runtime.performance_access().reset_counters();
    let live_batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint.clone(),
            64,
        ))
        .unwrap();
    let live_patch_batch = runtime
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(baseline.patch_position()),
            max_commits: 64,
        })
        .unwrap();
    let live_counters = runtime.performance_access().counters();

    let diff_digest = digest_patch_batch_surface(&live_patch_batch.patches);
    let cdc_digest = digest_subscriber_boundary_cdc_surface(
        &live_batch.patches,
        live_batch.continuation.crossed_boundaries(),
        live_batch.continuation.continuation_summary(),
        &live_batch.recovery_decision,
    );
    let cdc_diagnostics_digest = digest_diagnostics_batch_surface(&live_batch.diagnostics);
    let continuation_counter_snapshot = digest_subscriber_continuation_counter_pair(
        live_counters.subscriber_continue_visible_bridge_count,
        live_counters.schema_normalized_descriptor_compositions,
    );

    let (_recovery, recovered) = checkpoint_and_recover_with(&runtime, || {
        let registry = AspectSchemaFixture {
            schema_version_id: SchemaVersionId(3),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry();
        RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::CertificationCore)
            .schema_registry(registry)
            .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
            .durable_store_layout(DurableStoreLayout {
                root_path: unique_test_store_path("worth-relational-diff-cdc-truth-parity"),
                segment_commit_capacity: 2,
            })
            .build()
    });

    let recovered_batch = recovered
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            baseline_checkpoint,
            64,
        ))
        .unwrap();
    let recovered_patch_batch = recovered
        .publication()
        .read_patch_stream(PatchStreamRequest {
            after_position: Some(baseline.patch_position()),
            max_commits: 64,
        })
        .unwrap();
    let recovered_counters = recovered.performance_access().counters();

    assert_eq!(
        diff_digest,
        digest_patch_batch_surface(&recovered_patch_batch.patches)
    );
    assert_eq!(
        cdc_digest,
        digest_subscriber_boundary_cdc_surface(
            &recovered_batch.patches,
            recovered_batch.continuation.crossed_boundaries(),
            recovered_batch.continuation.continuation_summary(),
            &recovered_batch.recovery_decision,
        )
    );
    assert_eq!(
        cdc_diagnostics_digest,
        digest_diagnostics_batch_surface(&recovered_batch.diagnostics)
    );
    assert_eq!(
        continuation_counter_snapshot,
        digest_subscriber_continuation_counter_pair(
            recovered_counters.subscriber_continue_visible_bridge_count,
            recovered_counters.schema_normalized_descriptor_compositions,
        )
    );
    assert!(live_counters.replay_digest_parity_checks == 0);
    assert!(live_counters.subscriber_continue_visible_bridge_count >= 1);
    assert!(live_counters.schema_normalized_descriptor_compositions >= 1);
}
