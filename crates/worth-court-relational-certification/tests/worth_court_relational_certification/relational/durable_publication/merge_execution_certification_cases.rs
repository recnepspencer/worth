//! Three executed merge fixtures, each retaining its own medium, declared schema and outcome.
use super::branch_history_publications::{
    create_entity, create_entity_outcome_on_branch, fork, transaction, update_entity,
    update_entity_on_branch,
};
use super::merge_proof_journey::merge_request;
use super::merge_witness_schema::policy_builder;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    history::BranchId,
    identity::{KindId, PartitionId},
    merge::{AspectMergePolicyKind, MergeExecutionOutcome},
    runtime::{EntityReadRecord, RelationalRuntime},
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_key, field_key},
        publications::{string_aspect_field_patch, string_aspect_value},
    },
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};

/// One executed merge with everything its recovery certification needs: the
/// original medium, the constructor of its declared schema over a medium, the
/// source runtime and the original merge commit result.
pub(super) struct MergeCertificationCase {
    pub(super) medium: Arc<TestDurableMedium>,
    pub(super) declared_runtime: fn(&Arc<TestDurableMedium>) -> RelationalRuntime,
    pub(super) runtime: RelationalRuntime,
    pub(super) merge: MergeExecutionOutcome,
}

fn execute_feature_into_main(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    case: &str,
) -> MergeExecutionOutcome {
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(merge_request())
                .unwrap(),
        )
        .unwrap_or_else(|error| panic!("prepared {case} merge: {error:?}"));
    runtime
        .execute_prepared_merge(execution, prepared)
        .unwrap_or_else(|error| panic!("executed {case} merge: {error:?}"))
}

pub(super) fn exact_shared_merge_execution(
    execution: &ExecutionRequest<'_, '_>,
) -> MergeCertificationCase {
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let shared = create_entity(execution, &runtime, "shared");
    fork(execution, &runtime, "feature");
    update_entity(execution, &runtime, shared, "same");
    update_entity_on_branch(
        execution,
        &runtime,
        shared,
        "same",
        BranchId("feature".to_string()),
    );

    let merge = execute_feature_into_main(execution, &runtime, "exact-shared");

    assert_eq!(merge.structural_summary.preserved_shared_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0);

    MergeCertificationCase {
        medium,
        declared_runtime: durable_runtime,
        runtime,
        merge,
    }
}

pub(super) fn source_only_addition_merge_execution(
    execution: &ExecutionRequest<'_, '_>,
) -> MergeCertificationCase {
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let merge = execute_feature_into_main(execution, &runtime, "source-only");

    assert_eq!(merge.structural_summary.adopted_source_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_entity_create_count, 1);

    MergeCertificationCase {
        medium,
        declared_runtime: durable_runtime,
        runtime,
        merge,
    }
}

/// The certification-profile durable runtime over the name-identified schema
/// whose status aspect declares PreferRicher.
fn prefer_richer_runtime(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    policy_builder(AspectMergePolicyKind::PreferRicher, true)
        .profile(RelationalRuntimeProfile::CertificationCore)
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration")
}

pub(super) fn prefer_richer_merge_execution(
    execution: &ExecutionRequest<'_, '_>,
) -> MergeCertificationCase {
    let medium = TestDurableMedium::new();
    let runtime = prefer_richer_runtime(&medium);

    let main_entity = create_entity(execution, &runtime, "shared-name");
    fork(execution, &runtime, "feature");

    let mut feature_txn = transaction(&runtime, &BranchId("feature".to_string()));
    feature_txn
        .push_batch(
            WorkerIntentBatch::new("feature-seed").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("feature-shared"),
                    fields: string_aspect_field_patch([
                        (aspect_key("name"), field_key("name"), "shared-name"),
                        (aspect_key("status"), field_key("status"), "active"),
                    ]),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    feature_txn
        .commit(execution, &runtime)
        .expect("feature branch seed");

    let merge = execute_feature_into_main(execution, &runtime, "prefer-richer");

    assert_eq!(merge.structural_summary.reconciled_record_count, 1);
    assert_eq!(merge.structural_summary.emitted_entity_update_count, 1);
    let current = runtime
        .read_truth()
        .read_snapshot(&merge.commit.snapshot, execution)
        .expect("read request remains live")
        .expect("current merge snapshot");
    let current_record = current
        .get_entity(main_entity)
        .expect("merged target entity remains visible");
    assert_eq!(
        scalar_aspect_value(current_record, "name"),
        Some(&string_aspect_value("shared-name"))
    );
    assert_eq!(
        scalar_aspect_value(current_record, "status"),
        Some(&string_aspect_value("active"))
    );

    MergeCertificationCase {
        medium,
        declared_runtime: prefer_richer_runtime,
        runtime,
        merge,
    }
}

/// The authoritative scalar value of one declared single-field aspect.
fn scalar_aspect_value<'record>(
    record: &'record EntityReadRecord,
    aspect: &str,
) -> Option<&'record AspectValue> {
    let state = record.authoritative_aspect_state.as_ref()?;
    match state.get(&aspect_key(aspect))?.view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(_) => None,
    }
}
