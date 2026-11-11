use std::sync::Arc;

use super::{RelationalBranchRootCaptureDenial, RelationalBranchVisibilityCommitment};
use crate::history::data::{BranchId, CommitId};
use crate::schema::data::SchemaId;
use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};

#[path = "root_tests/request.rs"]
mod request;

fn committed_root_and_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::runtime::RelationalRuntime,
    Arc<super::RelationalBranchRoot>,
    crate::history::data::CanonicalCommitEnvelope,
) {
    let runtime = runtime_with_test_schema();
    let performed = create_entity_outcome(execution, &runtime, "branch-root-axis-binding");
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("performed commit installs one complete branch root");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, performed.commit.commit_id)
        .unwrap()
        .expect("performed commit owns one canonical envelope")
        .copy_original_for_test(execution);
    (runtime, root, envelope)
}

#[test]
fn relink_rejects_schema_authority_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut envelope) = committed_root_and_envelope(execution);
    let mut parts = (*envelope.schema_authority).clone();
    parts.primary_schema_id = Some(SchemaId("mutated-schema".to_owned()));
    envelope.body_mut_for_test().schema_authority = parts.into();

    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(envelope),
            )
            .unwrap(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .expect_err("canonical schema authority mutation cannot be readmitted");

    assert!(matches!(
        denial,
        RelationalBranchRootCaptureDenial::SchemaRootMismatch { .. }
    ));
}

#[test]
fn relink_rejects_visibility_tuple_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut envelope) = committed_root_and_envelope(execution);
    envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect ^= true;

    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(envelope),
            )
            .unwrap(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .expect_err("visibility tuple mutation cannot replace the committed envelope");

    assert!(matches!(
        denial,
        RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch { .. }
    ));
}

#[test]
fn completeness_recomputes_visibility_commitment() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut altered_envelope) = committed_root_and_envelope(execution);
    assert!(root.is_complete(execution, &runtime.services.symbols.interner_snapshot()));
    altered_envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect ^= true;
    let mut corrupted = root.as_ref().clone();
    let committed = corrupted
        .committed
        .as_mut()
        .expect("performed root owns committed axes");
    committed.axes.visibility = RelationalBranchVisibilityCommitment::for_root(
        &altered_envelope,
        committed.axes.storage_root,
        committed.axes.schema_root,
        committed.axes.correctness_index,
    );

    assert!(
        !corrupted.is_complete(execution, &runtime.services.symbols.interner_snapshot()),
        "completeness must reject a commitment to any different visible tuple"
    );
}

#[test]
fn visibility_commitment_binds_every_visible_tuple_axis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, envelope) = committed_root_and_envelope(execution);
    let storage_root = [17; 32];
    let schema_root = [29; 32];
    let baseline = RelationalBranchVisibilityCommitment::for_root(
        &envelope,
        storage_root,
        schema_root,
        super::RelationalRootCorrectnessIndex::AuthoritativeFallback,
    )
    .digest();
    assert_ne!(
        production_visibility(&envelope, [18; 32], schema_root),
        baseline
    );
    assert_ne!(
        production_visibility(&envelope, storage_root, [30; 32]),
        baseline
    );

    let mut mutants = Vec::new();
    let mut commit = envelope.copy_original_for_test(execution);
    commit.body_mut_for_test().commit.commit_id =
        CommitId(commit.body_mut_for_test().commit.commit_id.0 + 1);
    mutants.push(("commit", commit));
    let mut version = envelope.copy_original_for_test(execution);
    version.body_mut_for_test().commit.version_id.0 += 1;
    mutants.push(("version", version));
    let mut parents = envelope.copy_original_for_test(execution);
    parents.body_mut_for_test().commit.parents = vec![CommitId(41), CommitId(43)];
    mutants.push(("ordered parents", parents));
    let mut parents = envelope.copy_original_for_test(execution);
    parents.body_mut_for_test().commit.parents = vec![CommitId(43), CommitId(41)];
    mutants.push(("parent order", parents));
    let mut branch = envelope.copy_original_for_test(execution);
    branch.body_mut_for_test().branch_context = BranchId("visibility-mutant".to_owned());
    mutants.push(("branch", branch));
    let mut receipt_branch = envelope.copy_original_for_test(execution);
    receipt_branch.body_mut_for_test().commit.branch_id =
        BranchId("receipt-branch-mutant".to_owned());
    mutants.push(("receipt branch", receipt_branch));
    let mut checkpoint = envelope.copy_original_for_test(execution);
    checkpoint.body_mut_for_test().branch_cell_checkpoint = None;
    mutants.push(("branch checkpoint", checkpoint));
    let mut authority_kind = envelope.copy_original_for_test(execution);
    authority_kind.body_mut_for_test().authority_kind =
        crate::history::data::CanonicalCommitAuthorityKind::BranchReferenceMovement;
    mutants.push(("authority kind", authority_kind));
    let mut merge_parents = envelope.copy_original_for_test(execution);
    merge_parents
        .body_mut_for_test()
        .merge_parent_branches
        .push(BranchId("merge-mutant".to_owned()));
    mutants.push(("merge branches", merge_parents));
    let mut merge_bases = envelope.copy_original_for_test(execution);
    merge_bases
        .body_mut_for_test()
        .merge_base_commits
        .push(CommitId(47));
    mutants.push(("merge bases", merge_bases));
    let mut schema_version = envelope.copy_original_for_test(execution);
    schema_version.body_mut_for_test().schema_version.0 += 1;
    mutants.push(("schema version", schema_version));
    let mut schema_authority = envelope.copy_original_for_test(execution);
    let mut parts = (*schema_authority.schema_authority).clone();
    parts.primary_schema_id = Some(SchemaId("visibility-mutant".to_owned()));
    schema_authority.body_mut_for_test().schema_authority = parts.into();
    mutants.push(("schema authority", schema_authority));
    let mut plan = envelope.copy_original_for_test(execution);
    plan.body_mut_for_test().merged_plan.transaction_id.0 += 1;
    mutants.push(("merged plan", plan));
    let mut allocations = envelope.copy_original_for_test(execution);
    assert!(!allocations.record_allocations().is_empty());
    allocations
        .body_mut_for_test()
        .install_record_allocations(Vec::new());
    mutants.push(("record allocations", allocations));
    let mut patch = envelope.copy_original_for_test(execution);
    patch.body_mut_for_test().patch.authoritative_record_patches[0].contains_opaque_aspect ^= true;
    mutants.push(("patch", patch));
    let mut lineage = envelope.copy_original_for_test(execution);
    lineage
        .body_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id += 1;
    mutants.push(("lineage", lineage));
    let mut descriptor_semantics = envelope.copy_original_for_test(execution);
    descriptor_semantics
        .body_mut_for_test()
        .descriptor_semantics_version
        .0 += 1;
    mutants.push(("descriptor semantics", descriptor_semantics));

    for (axis, mutant) in mutants {
        assert_ne!(
            production_visibility(&mutant, storage_root, schema_root),
            baseline,
            "{axis} must change the visibility commitment"
        );
        assert!(
            root.relink_canonical_envelope(
                execution,
                crate::branch::RelationalOriginalRootCommitEvidence::prepare(
                    execution,
                    Arc::new(mutant)
                )
                .unwrap(),
                &runtime.services.symbols.interner_snapshot(),
            )
            .is_err(),
            "{axis} mutation must not relink to the committed root"
        );
    }
}

fn production_visibility(
    envelope: &crate::history::data::CanonicalCommitEnvelope,
    storage_root: [u8; 32],
    schema_root: [u8; 32],
) -> [u8; 32] {
    RelationalBranchVisibilityCommitment::for_root(
        envelope,
        storage_root,
        schema_root,
        super::RelationalRootCorrectnessIndex::AuthoritativeFallback,
    )
    .digest()
}
