use std::sync::Arc;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

use crate::branch::{RelationalBranchReferenceCell, RelationalBranchTarget};
use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::VersionId;
use crate::facade::merge::{MergeIntent, MergePlanningRequest};
use crate::facade::runtime::RelationalRuntime;
use crate::history::data::{MergeBaseSelectionRule, RelationalMergeBranchBasis, VersionNode};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    runtime_with_test_schema,
};

#[test]
fn merge_branch_basis_matches_planning_artifact_and_history_scope_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
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
        .resolve_merge_branch_basis(request.source_branch(), request.target_branch())
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

#[test]
fn merge_branch_basis_denial_happens_before_planning_for_disconnected_branch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    let main_branch = BranchId("main".to_string());
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
            MergePlanningRequest::new(main_branch, orphan_branch, MergeIntent::ReconcileIntoTarget,)
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

fn graft_disconnected_branch_head(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) {
    let disconnected = runtime_with_test_schema();
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

    let source_envelope = disconnected
        .history
        .recorded_commit_envelope(source_head.commit_id)
        .expect("source envelope");
    let mut envelope =
        crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test(
            &source_envelope,
            execution,
        );
    envelope.body_mut_for_test().commit = orphan_head.clone();
    envelope.body_mut_for_test().branch_context = branch_id.clone();

    let original =
        crate::branch::RelationalOriginalRootCommitEvidence::prepare(execution, Arc::new(envelope))
            .expect("disconnected commit evidence prepares");
    let artifact = Arc::new(
        crate::history::RelationalCommitArtifact::from_preencoded(Arc::clone(&original))
            .expect("disconnected commit artifact encodes"),
    );
    runtime
        .history
        .with_ledger_mut(|ledger| ledger.commit_catalog.append_preencoded_recovery(artifact))
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
    runtime.history.with_ledger_mut(|ledger| {
        ledger.commit_envelopes.insert(
            original.envelope().commit.commit_id,
            Arc::new(crate::history::RelationalCommitEnvelopeResidency::resident(
                original,
            )),
        )
    });
}
