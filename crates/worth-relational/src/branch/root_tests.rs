use crate::branch::root::identity::RelationalBranchRootIdentityIssuer;
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

use super::{RelationalBranchRootCaptureDenial, RelationalBranchVisibilityCommitment};
use crate::history::data::{BranchId, CommitId};
use crate::schema::data::SchemaId;

#[path = "root_tests/request.rs"]
mod request;

use super::physical_linkage_tests::{capture_committed_fixture, committed_root_and_envelope};

#[test]
fn publication_cost_distinguishes_new_and_reused_schema_authority() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = capture_committed_fixture(execution, 1, None);
    let second = capture_committed_fixture(execution, 2, Some(&first));

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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (symbols, root, mut envelope) = committed_root_and_envelope(execution);
    let mut parts = (*envelope.schema_authority).clone();
    parts.primary_schema_id = Some(SchemaId("mutated-schema".to_owned()));
    envelope.body_mut_for_test().schema_authority = parts.into();

    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::root::commit_evidence::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(envelope),
            )
            .unwrap(),
            &symbols,
        )
        .expect_err("canonical schema authority mutation cannot be readmitted");

    assert!(matches!(
        denial,
        RelationalBranchRootCaptureDenial::SchemaRootMismatch { .. }
    ));
}

#[test]
fn relink_rejects_visibility_tuple_mutation() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (symbols, root, mut envelope) = committed_root_and_envelope(execution);
    envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect ^= true;

    let denial = root
        .relink_canonical_envelope(
            execution,
            crate::branch::root::commit_evidence::RelationalOriginalRootCommitEvidence::prepare(
                execution,
                Arc::new(envelope),
            )
            .unwrap(),
            &symbols,
        )
        .expect_err("visibility tuple mutation cannot replace the committed envelope");

    assert!(matches!(
        denial,
        RelationalBranchRootCaptureDenial::VisibilityCommitmentMismatch { .. }
    ));
}

#[test]
fn completeness_recomputes_visibility_commitment() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (symbols, root, mut altered_envelope) = committed_root_and_envelope(execution);
    assert!(root.is_complete(execution, &symbols));
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
        !corrupted.is_complete(execution, &symbols),
        "completeness must reject a commitment to any different visible tuple"
    );
}

#[test]
fn visibility_commitment_binds_every_visible_tuple_axis() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (symbols, root, envelope) = committed_root_and_envelope(execution);
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
    let mut commit = envelope.independent_fixture_image();
    commit.body_mut_for_test().commit.commit_id =
        CommitId(commit.body_mut_for_test().commit.commit_id.0 + 1);
    mutants.push(("commit", commit));
    let mut version = envelope.independent_fixture_image();
    version.body_mut_for_test().commit.version_id.0 += 1;
    mutants.push(("version", version));
    let mut parents = envelope.independent_fixture_image();
    parents.body_mut_for_test().commit.parents = vec![CommitId(41), CommitId(43)];
    mutants.push(("ordered parents", parents));
    let mut parents = envelope.independent_fixture_image();
    parents.body_mut_for_test().commit.parents = vec![CommitId(43), CommitId(41)];
    mutants.push(("parent order", parents));
    let mut branch = envelope.independent_fixture_image();
    branch.body_mut_for_test().branch_context = BranchId("visibility-mutant".to_owned());
    mutants.push(("branch", branch));
    let mut receipt_branch = envelope.independent_fixture_image();
    receipt_branch.body_mut_for_test().commit.branch_id =
        BranchId("receipt-branch-mutant".to_owned());
    mutants.push(("receipt branch", receipt_branch));
    let mut checkpoint = envelope.independent_fixture_image();
    checkpoint.body_mut_for_test().branch_cell_checkpoint = None;
    mutants.push(("branch checkpoint", checkpoint));
    let mut authority_kind = envelope.independent_fixture_image();
    authority_kind.body_mut_for_test().authority_kind =
        crate::history::data::CanonicalCommitAuthorityKind::BranchReferenceMovement;
    mutants.push(("authority kind", authority_kind));
    let mut merge_parents = envelope.independent_fixture_image();
    merge_parents
        .body_mut_for_test()
        .merge_parent_branches
        .push(BranchId("merge-mutant".to_owned()));
    mutants.push(("merge branches", merge_parents));
    let mut merge_bases = envelope.independent_fixture_image();
    merge_bases
        .body_mut_for_test()
        .merge_base_commits
        .push(CommitId(47));
    mutants.push(("merge bases", merge_bases));
    let mut schema_version = envelope.independent_fixture_image();
    schema_version.body_mut_for_test().schema_version.0 += 1;
    mutants.push(("schema version", schema_version));
    let mut schema_authority = envelope.independent_fixture_image();
    let mut parts = (*schema_authority.schema_authority).clone();
    parts.primary_schema_id = Some(SchemaId("visibility-mutant".to_owned()));
    schema_authority.body_mut_for_test().schema_authority = parts.into();
    mutants.push(("schema authority", schema_authority));
    let mut plan = envelope.independent_fixture_image();
    plan.body_mut_for_test().merged_plan.transaction_id.0 += 1;
    mutants.push(("merged plan", plan));
    let mut allocations = envelope.independent_fixture_image();
    assert!(!allocations.record_allocations().is_empty());
    allocations
        .body_mut_for_test()
        .install_record_allocations(Vec::new());
    mutants.push(("record allocations", allocations));
    let mut patch = envelope.independent_fixture_image();
    patch.body_mut_for_test().patch.authoritative_record_patches[0].contains_opaque_aspect ^= true;
    mutants.push(("patch", patch));
    let mut lineage = envelope.independent_fixture_image();
    lineage
        .body_mut_for_test()
        .published_lineage_mut_for_test()
        .lineage_events_mut()[0]
        .event_id += 1;
    mutants.push(("lineage", lineage));
    let mut descriptor_semantics = envelope.independent_fixture_image();
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
                crate::branch::root::commit_evidence::RelationalOriginalRootCommitEvidence::prepare(
                    execution,
                    Arc::new(mutant)
                )
                .unwrap(),
                &symbols,
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

#[test]
fn request_root_refusal_preserves_original_and_releases_partial_backing() {
    let host = crate::execution::test_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, original, _) = committed_root_and_envelope(&execution);
    let axes = original.axes();
    let issuer = RelationalBranchRootIdentityIssuer::default();
    let calibration = SerialMemoryBudget::new(Some(1024 * 1024));
    let host =
        SerialRequest::from_memory(calibration.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&host);
    let mut controller = 0;
    execution
        .run_native_with_checkpoint(|_| controller = calibration.charged_memory_bytes())
        .unwrap();
    let budget = SerialMemoryBudget::new(Some(
        controller + size_of::<super::RelationalBranchRoot>() as u64 - 1,
    ));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let attempt = || {
        super::RelationalBranchRoot::readmit_unchanged_for_request(
            &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
            &issuer,
            &original,
            original.canonical_payload().unwrap(),
            original.descriptor().unwrap().clone(),
            original.retained_schema_authority(),
            original.output_publication_positions.clone(),
            &execution,
        )
    };
    assert!(matches!(
        attempt(),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(matches!(
        attempt(),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(original.axes(), axes);
}

#[test]
fn streamed_visibility_preserves_original_buffered_preimage_and_request_stop() {
    use sha2::{Digest, Sha256};
    let host = crate::execution::test_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, original, envelope) = committed_root_and_envelope(&execution);
    let axes = original.axes().unwrap();
    let mut reference = Sha256::new();
    reference.update(b"worth.relational.branch-visibility.v1\0");
    reference.update(axes.storage_root);
    reference.update(axes.schema_root);
    reference.update([0]);
    reference.update(envelope.encode_authoritative_payload().unwrap());
    let expected: [u8; 32] = reference.finalize().into();
    assert_eq!(expected, axes.visibility.digest());
    assert_eq!(
        RelationalBranchVisibilityCommitment::for_root_for_request(
            &envelope,
            axes.storage_root,
            axes.schema_root,
            axes.correctness_index,
            &execution,
        )
        .unwrap()
        .digest(),
        expected
    );
    let cancellation = CancellationSource::new();
    let host =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        RelationalBranchVisibilityCommitment::for_root_for_request(
            &envelope,
            axes.storage_root,
            axes.schema_root,
            axes.correctness_index,
            &ExecutionRequest::serial(&host),
        ),
        Err(RelationalBranchRootCaptureDenial::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::Cancelled
        ))
    ));
}
