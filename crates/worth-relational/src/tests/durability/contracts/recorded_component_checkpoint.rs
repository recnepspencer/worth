//! Exact component obligations survive a cold checkpoint without eager history.
use super::associated_effects::perform;
use crate::facade::branch::RecordedRelationalComponentReference as Recorded;
use crate::facade::durability::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
use crate::facade::mvcc::RelationalCallerAssociation as Association;
use crate::tests::support::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::canonical_runtime_frame::RelationalBranchBasisDescriptorOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeOwnerPort as _;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointFieldsFixturePort as _;

fn current(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> Recorded {
    let port = runtime.owner_component_services().basis_port();
    let (_, basis) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    port.export_recorded_current_component(execution, &basis)
        .unwrap()
}

fn capture(
    execution: &ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
) -> crate::durability::data::RelationalNativeCheckpoint {
    source
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap()
}

struct Original {
    medium: Arc<OwnedTestDurableMedium>,
    source: RelationalRuntime,
    before: Recorded,
    after: Recorded,
    association: Association,
    unrelated: crate::publication::patch::data::PatchStreamPosition,
    unrelated_id: crate::history::data::CommitId,
}

fn original(execution: &ExecutionRequest<'_, '_>, empty: bool) -> Original {
    let medium = OwnedTestDurableMedium::new();
    let source = durable_runtime_for_medium(&medium);
    if !empty {
        create_entity(execution, &source, "original-predecessor");
    }
    let before = current(execution, &source);
    let (_, basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let before_lease = source.retain_component_basis(&basis).unwrap();
    drop(basis);
    let association = Association::from_bytes(b"checkpoint-original-external-custody").unwrap();
    let performed = perform(execution, &source, "original-successor", &association);
    let after_basis = performed.next_basis().clone();
    let result = source.settle_performed_publication(performed).unwrap();
    release_test_commit_snapshot(&source, &result);
    let after = source
        .owner_component_services()
        .basis_port()
        .export_recorded_retained_component(execution, &after_basis)
        .unwrap();
    let after_lease = source.retain_component_basis(&after_basis).unwrap();
    drop(after_basis);
    create_entity(execution, &source, "unrelated-body");
    let unrelated_artifact = medium.latest_committed_artifact().unwrap();
    let unrelated = unrelated_artifact.position();
    let unrelated_id = unrelated_artifact.commit_id();
    create_entity(execution, &source, "checkpoint-current-head");
    let checkpoint = capture(execution, &source);
    // The original unique leases may disappear after capture without weakening
    // the exact header/member custody already captured from those observations.
    source.release_component_basis(before_lease).unwrap();
    source.release_component_basis(after_lease).unwrap();
    medium.file_checkpoint(checkpoint);
    Original {
        medium,
        source,
        before,
        after,
        association,
        unrelated,
        unrelated_id,
    }
}

fn recover(
    execution: &ExecutionRequest<'_, '_>,
    medium: &Arc<OwnedTestDurableMedium>,
) -> (
    RelationalRuntime,
    crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority,
) {
    let mut runtime = durable_runtime_for_medium(medium);
    let plan = medium
        .recovery_plan(
            execution,
            &runtime,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    (runtime, authority)
}

fn original_custody(empty: bool, retire: bool) {
    let stop = worth_execution::CancellationSource::new();
    let host = crate::test_execution_policy::serial_request().with_cancellation(stop.token());
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, empty);
    let offers = original.medium.offered_count();
    let (runtime, authority) = recover(execution, &original.medium);
    let unrelated_sha = original.medium.drop_artifact_and_member(original.unrelated);
    original.medium.start_recording_member_reads();
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(execution, &original.association)
        .unwrap()
        .pop()
        .unwrap();
    assert!(
        original.medium.non_index_member_read_digests().is_empty(),
        "discovery must perform zero canonical body reads"
    );
    let selected = runtime
        .history
        .commit_artifact(subject.receipt().commit_id)
        .unwrap()
        .envelope_residency();
    assert!(selected.positioned_if_resident().is_none());
    let selected_sha = original.medium.artifact_sha256(subject.position());
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    assert_eq!(
        original.medium.non_index_member_read_digests(),
        vec![selected_sha],
        "readmission must read only the selected commit member"
    );
    let semantic = selected
        .demand(execution, 1 << 30)
        .unwrap()
        .semantic_result_arc();
    assert_eq!(effect.semantic_result(), semantic.as_ref());
    let proof = authority
        .admit_recorded_publication(
            execution,
            &runtime,
            effect,
            &original.before,
            &original.after,
        )
        .unwrap();
    assert_eq!(
        proof.custody().effect().semantic_result(),
        semantic.as_ref()
    );
    // The same shared Native owner performs maintenance while real custody
    // pins preserve both original roots.
    let shared: &RelationalRuntime = &runtime;
    shared.run_branch_root_reclamation_pass();
    for (basis, recorded) in [
        (proof.custody().predecessor(), &original.before),
        (proof.custody().successor(), &original.after),
    ] {
        assert_eq!(
            &basis.descriptor().in_canonical_frame().unwrap(),
            recorded.descriptor()
        );
        basis
            .demand_all_full(execution, &runtime.services.symbols)
            .unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    }
    assert!(runtime
        .compare_current_exact(proof.custody().successor())
        .is_err());
    assert_eq!(
        original.medium.offered_count(),
        offers,
        "recovery never republishes"
    );
    assert!(runtime
        .history
        .commit_artifact(original.unrelated_id)
        .unwrap()
        .envelope_residency()
        .positioned_if_resident()
        .is_none());
    if retire {
        use crate::change_source::RelationalRetainedObservationRetirementProgress as Progress;
        let basis = proof.custody().predecessor().clone();
        let terminal = runtime
            .services
            .instrumentation
            .external_retention_terminal_accounting();
        let before = terminal.snapshot();
        let mut task = proof.consume(|original| original.into_retirement());
        crate::history::retention::assert_recovered_cut_and_lane_busy_for_test(&basis, || {
            task.advance(execution)
        });
        drop(basis);
        stop.cancel();
        assert!(matches!(task.advance(execution), Progress::Stopped(_)));
        assert_eq!(
            terminal.snapshot().explicit_releases,
            before.explicit_releases + 1
        );
        assert_eq!(
            terminal.snapshot().dropped_releases,
            before.dropped_releases
        );
        let mut task = task.into_retired().unwrap_err();
        let retry = crate::test_execution_policy::serial_request();
        let retry = ExecutionRequest::serial(&retry);
        let retired = loop {
            match task.advance(&retry) {
                Progress::Advanced => continue,
                Progress::Complete => break task.into_retired().unwrap(),
                other => panic!("original recovered custody retirement: {other:?}"),
            }
        };
        assert_eq!(
            terminal.snapshot().explicit_releases,
            before.explicit_releases + 2
        );
        assert_eq!(
            terminal.snapshot().dropped_releases,
            before.dropped_releases
        );
        runtime.prepare_close().unwrap().finish();
        retired.reclaim();
    } else {
        drop(proof);
        runtime.prepare_close().unwrap().finish();
    }
    assert!(
        !original
            .medium
            .member_read_digests()
            .contains(&unrelated_sha),
        "unrelated commit member must never be read through retirement and close"
    );
}

#[test]
fn recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor() {
    original_custody(false, false);
}

#[test]
fn recorded_component_checkpoint_readmits_original_empty_predecessor() {
    original_custody(true, false);
}

#[test]
fn recorded_component_checkpoint_omits_released_components_and_plain_observations() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let checkpoint = capture(execution, &original.source);
    original.medium.file_checkpoint(checkpoint.clone());
    let source = original.source.durability.cold_source().unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
        execution,
        checkpoint.bytes(),
        source,
    )
    .unwrap();
    let directory = decoded
        .checkpoint
        .into_selected_directory_for_test()
        .unwrap();
    assert_eq!(directory.manifest.roots.len(), 1);
    assert!(directory.manifest.retained_empty_components.is_empty());
    assert!(!directory.manifest.roots.iter().any(|root| {
        root.recorded_components
            .contains(original.before.descriptor())
            || root
                .recorded_components
                .contains(original.after.descriptor())
    }));

    // An ordinary observation has no external durable component obligation.
    let (_, observed) = original
        .source
        .observe_branch(&original.source.main_branch_identity())
        .unwrap();
    create_entity(
        execution,
        &original.source,
        "observation-alone-is-not-a-component",
    );
    let next = capture(execution, &original.source);
    original.medium.file_checkpoint(next.clone());
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
        execution,
        next.bytes(),
        original
            .source
            .durability
            .cold_source()
            .expect("durable source owns cold source"),
    )
    .unwrap();
    assert_eq!(
        decoded
            .checkpoint
            .into_selected_directory_for_test()
            .unwrap()
            .manifest
            .roots
            .len(),
        1
    );
    drop(observed);
}

#[test]
fn recorded_component_checkpoint_refuses_chk15_before_source_admission() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
    let checkpoint = capture(execution, &runtime);
    let mut bytes = checkpoint.bytes().to_vec();
    let old = b"WORTH-REL-CHK15\n";
    bytes[..old.len()].copy_from_slice(old);
    let stop = crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
        execution,
        &bytes,
        crate::tests::support::durability::empty_cold_source(),
    )
    .unwrap_err();
    assert_eq!(
        expect_durability_failure(stop).class,
        crate::durability::data::RecoveryFailureClass::UnsupportedCheckpointVersion {
            found: 15,
            supported: 24,
        }
    );
}

#[test]
fn recorded_original_retirement_preserves_both_leases_across_busy_and_stop() {
    original_custody(false, true);
}
