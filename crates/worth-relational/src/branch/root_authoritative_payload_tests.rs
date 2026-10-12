use std::sync::Arc;

use super::{RelationalBranchRoot, RelationalBranchRootCaptureDenial};
use crate::config::data::CascadeDeletePolicy;
use crate::history::data::BranchId;
use crate::schema::data::SchemaVersionId;
use crate::tests::support::*;

#[test]
fn strategy_artifacts_are_bound_into_the_committed_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = crate::commit_strategies::facade::persisted_intent_runtime(
        unique_test_store_path("root-strategy-artifact"),
        true,
    );
    let entity = create_entity(execution, &runtime, "strategy-before");
    let outcome = crate::commit_strategies::facade::execute_persisted_intent_strategy_commit(
        execution,
        &mut runtime,
        entity,
    );
    let (root, mut envelope) = committed_root_and_envelope(&runtime, outcome.commit.commit_id);
    assert!(envelope.strategy_artifacts.take().is_some());

    assert_axis_mutation_rejected(&runtime, &root, envelope, "strategy artifacts");
}

#[test]
fn merge_execution_authority_is_bound_into_the_committed_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "merge-main");
    runtime
        .history_authority()
        .fork_branch_from(BranchId("feature".to_owned()), &BranchId("main".to_owned()))
        .expect("feature branch forks");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "merge-feature",
        BranchId("feature".to_owned()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            crate::merge::data::MergeExecutionRequest::new(
                BranchId("main".to_owned()),
                BranchId("feature".to_owned()),
                crate::merge::data::MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("governed merge prepares");
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("governed merge executes");
    let (root, mut envelope) =
        committed_root_and_envelope(&runtime, outcome.commit.commit.commit_id);
    assert!(envelope.merge_execution_authority.take().is_some());

    assert_axis_mutation_rejected(&runtime, &root, envelope, "merge execution authority");
}

#[test]
fn schema_transition_descriptors_are_each_bound_into_the_committed_root() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = persisted_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "schema-before");
    runtime.set_schema_registry_for_test(
        AspectSchemaFixture {
            schema_version_id: SchemaVersionId(2),
            ..AspectSchemaFixture::with_default_declared_aspects(
                CascadeDeletePolicy::CascadeDeleteRelations,
            )
        }
        .build_registry(),
    );
    let input = test_owner_transaction_validation_input_for_main(&runtime).with_schema_transition(
        schema_transition(),
        Some(crate::schema::data::SchemaReconciliationPolicy::PreserveInformation),
    );
    let mut transaction = runtime
        .begin_branch_transaction(input.basis(), input.intent().clone())
        .expect("schema transition transaction binds");
    transaction
        .push_batch(
            batch_create("schema-after"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let outcome = transaction
        .commit(execution, &runtime)
        .expect("schema transition commits");
    let (root, envelope) = committed_root_and_envelope(&runtime, outcome.commit.commit_id);

    let mut transition = envelope.clone();
    assert!(transition.schema_transition.take().is_some());
    assert_axis_mutation_rejected(&runtime, &root, transition, "schema transition");
    let mut continuation = envelope.clone();
    assert!(continuation.schema_continuation_descriptor.take().is_some());
    assert_axis_mutation_rejected(&runtime, &root, continuation, "continuation descriptor");
    let mut reconciliation = envelope;
    assert!(reconciliation
        .schema_reconciliation_descriptor
        .take()
        .is_some());
    assert_axis_mutation_rejected(&runtime, &root, reconciliation, "reconciliation descriptor");
}

fn committed_root_and_envelope(
    runtime: &crate::runtime::RelationalRuntime,
    commit_id: crate::history::data::CommitId,
) -> (
    Arc<RelationalBranchRoot>,
    crate::history::data::CanonicalCommitEnvelope,
) {
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("commit installs one complete main root");
    let envelope = root
        .canonical_envelope()
        .expect("root carries canonical envelope");
    assert_eq!(envelope.commit.commit_id, commit_id);
    (Arc::clone(&root), envelope.as_ref().clone())
}

fn assert_axis_mutation_rejected(
    runtime: &crate::runtime::RelationalRuntime,
    root: &Arc<RelationalBranchRoot>,
    mutant: crate::history::data::CanonicalCommitEnvelope,
    axis: &str,
) {
    let denial = root
        .relink_canonical_envelope(
            Arc::new(mutant),
            &runtime.services.symbols.interner_snapshot(),
        )
        .expect_err("authoritative payload mutation cannot relink");
    assert!(
        matches!(
            denial,
            RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch { .. }
        ),
        "{axis} mutation must change the committed root"
    );
}

fn schema_transition() -> crate::schema::data::ProposedSchemaTransition {
    use crate::schema::data::*;
    ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_owned()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("test".to_owned()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_owned()),
                SchemaVersionId(2),
                Some(crate::identity::data::KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: crate::tests::support::field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}
