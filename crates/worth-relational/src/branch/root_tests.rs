use std::sync::Arc;

use super::{RelationalBranchRootCaptureDenial, RelationalBranchVisibilityCommitment};
use crate::history::data::{BranchId, CommitId};
use crate::schema::data::SchemaId;
use crate::tests::support::{create_entity_outcome, durable_runtime_with_test_schema};

fn committed_root_and_envelope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    crate::runtime::RelationalRuntime,
    Arc<super::RelationalBranchRoot>,
    crate::history::data::CanonicalCommitEnvelope,
) {
    let runtime = durable_runtime_with_test_schema();
    let performed = create_entity_outcome(execution, &runtime, "branch-root-axis-binding");
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("performed commit installs one complete branch root");
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(performed.commit.commit_id)
        .expect("performed commit owns one canonical envelope")
        .clone();
    (runtime, root, envelope)
}

#[test]
fn publication_cost_distinguishes_new_and_reused_schema_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "schema-authority-allocation-owner");
    let first = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("first commit installs a root");
    create_entity_outcome(execution, &runtime, "schema-authority-reuse");
    let second = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .expect("second commit installs a root");

    assert_eq!(first.publication_cost().new_schema_authorities, 1);
    assert_eq!(first.publication_cost().reused_schema_authorities, 0);
    assert_eq!(second.publication_cost().new_schema_authorities, 0);
    assert_eq!(second.publication_cost().reused_schema_authorities, 1);
    assert_eq!(
        first.schema_authority().allocation_id(),
        second.schema_authority().allocation_id()
    );
}

#[test]
fn relink_rejects_schema_authority_mutation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, root, mut envelope) = committed_root_and_envelope(execution);
    envelope.schema_authority.primary_schema_id = Some(SchemaId("mutated-schema".to_owned()));

    let denial = root
        .relink_canonical_envelope(
            Arc::new(envelope),
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
    envelope.patch.authoritative_record_patches[0].contains_opaque_aspect ^= true;

    let denial = root
        .relink_canonical_envelope(
            Arc::new(envelope),
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
    assert!(root.is_complete(&runtime.services.symbols.interner_snapshot()));
    altered_envelope.patch.authoritative_record_patches[0].contains_opaque_aspect ^= true;
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
        !corrupted.is_complete(&runtime.services.symbols.interner_snapshot()),
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
    let mut commit = envelope.clone();
    commit.commit.commit_id = CommitId(commit.commit.commit_id.0 + 1);
    mutants.push(("commit", commit));
    let mut version = envelope.clone();
    version.commit.version_id.0 += 1;
    mutants.push(("version", version));
    let mut parents = envelope.clone();
    parents.commit.parents = vec![CommitId(41), CommitId(43)];
    mutants.push(("ordered parents", parents.clone()));
    parents.commit.parents.reverse();
    mutants.push(("parent order", parents));
    let mut branch = envelope.clone();
    branch.branch_context = BranchId("visibility-mutant".to_owned());
    mutants.push(("branch", branch));
    let mut receipt_branch = envelope.clone();
    receipt_branch.commit.branch_id = BranchId("receipt-branch-mutant".to_owned());
    mutants.push(("receipt branch", receipt_branch));
    let mut checkpoint = envelope.clone();
    checkpoint.branch_cell_checkpoint = None;
    mutants.push(("branch checkpoint", checkpoint));
    let mut authority_kind = envelope.clone();
    authority_kind.authority_kind =
        crate::history::data::CanonicalCommitAuthorityKind::BranchReferenceMovement;
    mutants.push(("authority kind", authority_kind));
    let mut merge_parents = envelope.clone();
    merge_parents
        .merge_parent_branches
        .push(BranchId("merge-mutant".to_owned()));
    mutants.push(("merge branches", merge_parents));
    let mut merge_bases = envelope.clone();
    merge_bases.merge_base_commits.push(CommitId(47));
    mutants.push(("merge bases", merge_bases));
    let mut schema_version = envelope.clone();
    schema_version.schema_version.0 += 1;
    mutants.push(("schema version", schema_version));
    let mut schema_authority = envelope.clone();
    schema_authority.schema_authority.primary_schema_id =
        Some(SchemaId("visibility-mutant".to_owned()));
    mutants.push(("schema authority", schema_authority));
    let mut plan = envelope.clone();
    plan.merged_plan.transaction_id.0 += 1;
    mutants.push(("merged plan", plan));
    let mut allocations = envelope.clone();
    assert!(!allocations.record_allocations().is_empty());
    allocations.install_record_allocations(Vec::new());
    mutants.push(("record allocations", allocations));
    let mut patch = envelope.clone();
    patch.patch.authoritative_record_patches[0].contains_opaque_aspect ^= true;
    mutants.push(("patch", patch));
    let mut lineage = envelope.clone();
    lineage
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id += 1;
    mutants.push(("lineage", lineage));
    let mut descriptor_semantics = envelope.clone();
    descriptor_semantics.descriptor_semantics_version.0 += 1;
    mutants.push(("descriptor semantics", descriptor_semantics));

    for (axis, mutant) in mutants {
        assert_ne!(
            production_visibility(&mutant, storage_root, schema_root),
            baseline,
            "{axis} must change the visibility commitment"
        );
        assert!(
            root.relink_canonical_envelope(
                Arc::new(mutant),
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
