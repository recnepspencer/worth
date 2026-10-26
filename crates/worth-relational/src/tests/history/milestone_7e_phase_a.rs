use std::sync::Arc;

use crate::branch::{RelationalBranchReferenceCell, RelationalBranchTarget};
use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::VersionId;
use crate::facade::merge::{MergeIntent, MergePlanningRequest};
use crate::facade::runtime::RelationalRuntime;
use crate::history::data::{MergeBaseSelectionRule, RelationalMergeBranchBasis, VersionNode};
use crate::tests::support::{
    checkpoint_and_recover_with, create_branch_from_main, create_entity,
    create_entity_outcome_on_branch, durable_runtime_with_test_schema,
};
use crate::transactions::data::MergeExecutionSummary;

#[test]
fn merge_branch_basis_matches_planning_artifact_and_history_scope_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let request = MergePlanningRequest::new(
        BranchId("main".to_string()),
        BranchId("feature".to_string()),
        MergeIntent::ReconcileIntoTarget,
    );

    let direct_basis = runtime
        .history()
        .resolve_merge_branch_basis(&request.source_branch, &request.target_branch)
        .expect("direct branch basis");
    let history_plan = runtime
        .merge()
        .plan_history_scope_for_test(execution, request.clone())
        .expect("history-scoped merge plan");
    let artifact = runtime
        .merge()
        .inspect_planning_scope(execution, request)
        .expect("planning artifact");

    assert_exact_branch_basis(&history_plan.basis, &direct_basis);
    assert_exact_branch_basis(&artifact.branch_basis, &direct_basis);
    assert_eq!(
        artifact.ancestry.merge_base_commit_id,
        artifact.branch_basis.merge_base().commit().commit_id
    );
    assert_eq!(
        artifact.branch_basis.merge_base().commit().commit_id,
        artifact.merge_base().commit().commit_id
    );
}

#[test]
fn merge_branch_basis_denial_happens_before_planning_for_missing_or_disconnected_branches() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    let main_branch = BranchId("main".to_string());
    let missing_branch = BranchId("missing".to_string());

    let missing_source = runtime
        .history()
        .resolve_merge_branch_basis(&missing_branch, &main_branch)
        .expect_err("missing source branch basis denial");
    assert!(matches!(
        missing_source,
        crate::history::data::RelationalMergeBranchBasisDenial::MissingSourceHead { branch_id }
            if branch_id == missing_branch
    ));
    assert!(matches!(
        runtime
            .merge()
            .inspect_planning_scope(execution, MergePlanningRequest::new(
                main_branch.clone(),
                missing_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            )),
        Err(crate::merge::data::MergePlanningError::MissingSourceHead { branch_id })
            if branch_id == missing_branch
    ));

    let orphan_branch = BranchId("orphan".to_string());
    graft_disconnected_branch_head(execution, &runtime, &orphan_branch);
    let missing_merge_base = runtime
        .history()
        .resolve_merge_branch_basis(&orphan_branch, &main_branch)
        .expect_err("disconnected branch basis denial");
    assert!(matches!(
        missing_merge_base,
        crate::history::data::RelationalMergeBranchBasisDenial::MissingMergeBase {
            source_branch,
            target_branch,
        } if source_branch == orphan_branch && target_branch == main_branch
    ));
    assert_eq!(
        runtime.merge().inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                main_branch.clone(),
                orphan_branch,
                MergeIntent::ReconcileIntoTarget,
            )
        ),
        Err(
            crate::merge::data::MergePlanningError::RequestNormalization(
                crate::merge::data::RelationalMergeRequestNormalizationDenial::OwnerBinding(
                    crate::merge::data::RelationalMergeRequestBindingDenial::IdentityMismatch,
                ),
            )
        ),
        "a descriptive orphan with no coherent exact root is denied before ancestry planning",
    );
}

#[test]
fn merge_branch_basis_survives_published_merge_outcome_and_durability_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            crate::facade::merge::MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    let live_envelope = runtime
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("live merge envelope");
    let live_authority = live_envelope
        .merge_execution_authority
        .clone()
        .expect("live merge authority");

    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("recovered merge envelope");
    let recovered_authority = recovered_envelope
        .merge_execution_authority
        .clone()
        .expect("recovered merge authority");

    assert_exact_branch_basis(
        &merge.execution_summary.branch_basis,
        &live_authority.execution_summary.branch_basis,
    );
    assert_exact_branch_basis(
        &merge.execution_summary.branch_basis,
        &recovered_authority.execution_summary.branch_basis,
    );
    assert_summary_projects_exact_basis(&merge.execution_summary);
    assert_summary_projects_exact_basis(&live_authority.execution_summary);
    assert_summary_projects_exact_basis(&recovered_authority.execution_summary);
    assert_eq!(live_authority, recovered_authority);
}

fn assert_exact_branch_basis(
    actual: &RelationalMergeBranchBasis,
    expected: &RelationalMergeBranchBasis,
) {
    assert_eq!(actual, expected);
    assert_eq!(actual.basis_digest(), expected.basis_digest());
    assert_eq!(actual.source_branch(), expected.source_branch());
    assert_eq!(actual.target_branch(), expected.target_branch());
    assert_eq!(actual.source_head(), expected.source_head());
    assert_eq!(actual.target_head(), expected.target_head());
    assert_eq!(
        actual.merge_base().rule(),
        MergeBaseSelectionRule::MaxCommitIdCommonAncestor
    );
    assert_eq!(actual.merge_base().rule(), expected.merge_base().rule());
    assert_eq!(actual.merge_base().commit(), expected.merge_base().commit());
    assert_eq!(
        actual.merge_base().supporting_left_ancestors(),
        expected.merge_base().supporting_left_ancestors()
    );
    assert_eq!(
        actual.merge_base().supporting_right_ancestors(),
        expected.merge_base().supporting_right_ancestors()
    );
}

fn assert_summary_projects_exact_basis(summary: &MergeExecutionSummary) {
    assert_eq!(
        summary.target_head_commit_id,
        summary.branch_basis.target_head().commit_id
    );
    assert_eq!(
        summary.source_head_commit_id,
        summary.branch_basis.source_head().commit_id
    );
    assert_eq!(
        summary.merge_base_commit_id,
        summary.branch_basis.merge_base().commit().commit_id
    );
}

fn graft_disconnected_branch_head(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) {
    let disconnected = durable_runtime_with_test_schema();
    create_entity(execution, &disconnected, "orphan-root");

    let source_head = disconnected
        .history()
        .branch_head(&BranchId("main".to_string()))
        .expect("disconnected source head");
    let mut orphan_head = source_head.clone();
    orphan_head.commit_id = CommitId(9_999_001);
    orphan_head.version_id = VersionId(9_999_001);
    orphan_head.branch_id = branch_id.clone();
    orphan_head.parents.clear();

    let mut envelope = disconnected
        .history
        .recorded_commit_envelope(source_head.commit_id)
        .expect("source envelope")
        .as_ref()
        .clone();
    envelope.commit = orphan_head.clone();
    envelope.branch_context = branch_id.clone();

    runtime
        .history
        .with_ledger_mut(|ledger| {
            ledger
                .commit_catalog
                .append_envelope(Arc::new(envelope.clone()))
        })
        .expect("disconnected artifact id is unique");
    let branch_cell =
        RelationalBranchReferenceCell::empty(runtime.runtime_instance_id(), branch_id.clone())
            .expect("disconnected branch identity is valid");
    branch_cell
        .advance_truth(worth_foundational::FoundationalBranchTarget::basis(
            RelationalBranchTarget::from_commit_receipt(
                runtime.runtime_instance_id(),
                &orphan_head,
                RelationalBranchTarget::roots_for_commit(&orphan_head),
            ),
        ))
        .expect("disconnected branch reference is valid");
    runtime.history.insert_branch_cell(branch_cell);
    runtime.history.insert_commit_graph_node(
        orphan_head.commit_id,
        VersionNode {
            commit: orphan_head,
        },
    );
    let digest = envelope.authoritative_payload_digest().unwrap();
    runtime.history.with_ledger_mut(|ledger| {
        ledger.commit_envelopes.insert(
            envelope.commit.commit_id,
            Arc::new(crate::history::RelationalCommitEnvelopeResidency::resident(
                Arc::new(envelope),
                digest,
            )),
        )
    });
}
