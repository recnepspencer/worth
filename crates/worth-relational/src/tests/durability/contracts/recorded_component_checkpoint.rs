//! Exact component obligations survive a cold checkpoint without eager history.
use super::associated_effects::perform;
fn current(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) -> Recorded {
    let port = runtime.owner_component_services().basis_port();
    let (_, basis) = port
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    port.export_recorded_current_component(execution, &basis)
        .unwrap()
}
use crate::facade::branch::RecordedRelationalComponentReference as Recorded;
use crate::facade::durability::{RecoveryVerificationMode, RelationalCheckpointArtifactPolicy};
use crate::facade::mvcc::RelationalCallerAssociation as Association;
use crate::tests::support::*;
use worth_execution::ExecutionRequest;

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
    source: RelationalRuntime,
    before: Recorded,
    after: Recorded,
    association: Association,
    unrelated: crate::publication::patch::data::PatchStreamPosition,
    unrelated_id: crate::history::data::CommitId,
}
fn original(execution: &ExecutionRequest<'_, '_>, empty: bool) -> Original {
    original_with_retained_successor(execution, empty, true)
}
fn original_with_retained_successor(
    execution: &ExecutionRequest<'_, '_>,
    empty: bool,
    retain_after: bool,
) -> Original {
    let source = durable_runtime_with_test_schema();
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
    let after_lease = retain_after.then(|| source.retain_component_basis(&after_basis).unwrap());
    drop(after_basis);
    create_entity(execution, &source, "unrelated-body");
    let unrelated_artifact = attached_test_medium(&source).offered().pop().unwrap();
    let unrelated = unrelated_artifact.position();
    let unrelated_id = unrelated_artifact.commit_id();
    create_entity(execution, &source, "checkpoint-current-head");
    let checkpoint = capture(execution, &source);
    // The original unique leases may disappear after capture without weakening
    // the exact header/member custody already captured from those observations.
    source.release_component_basis(before_lease).unwrap();
    if let Some(lease) = after_lease {
        source.release_component_basis(lease).unwrap();
    }
    attached_test_medium(&source).file_checkpoint(checkpoint);
    Original {
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
    source: &RelationalRuntime,
) -> (
    RelationalRuntime,
    crate::durability::authority::RecoveredRelationalRuntimeAuthority,
) {
    let mut runtime = durable_runtime_with_test_schema();
    let plan = medium_recovery_plan(
        execution,
        source,
        &mut runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let (_, authority) = runtime
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    (runtime, authority)
}
fn original_custody(empty: bool) {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, empty);
    let offers = attached_test_medium(&original.source).offered().len();
    let (runtime, authority) = recover(execution, &original.source);
    let medium = attached_test_medium(&runtime);
    medium.drop_artifact(original.unrelated);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(execution, &original.association)
        .unwrap()
        .pop()
        .unwrap();
    let selected = runtime
        .history
        .commit_artifact(subject.receipt().commit_id)
        .unwrap()
        .envelope_residency();
    assert!(selected.positioned_if_resident().is_none());
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(execution, &subject)
        .unwrap();
    let semantic = effect.semantic_result().clone();
    let proof = authority
        .admit_recorded_publication(
            execution,
            &runtime,
            effect,
            &original.before,
            &original.after,
        )
        .unwrap();
    assert_eq!(proof.custody().effect().semantic_result(), &semantic);
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
            .inner
            .root
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
    assert_eq!(medium.offered().len(), offers, "recovery never republishes");
    assert!(runtime
        .history
        .commit_artifact(original.unrelated_id)
        .unwrap()
        .envelope_residency()
        .positioned_if_resident()
        .is_none());
    drop(proof);
    runtime.prepare_close().unwrap().finish();
}
#[test]
fn recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor() {
    original_custody(false);
}
#[test]
fn recorded_component_checkpoint_readmits_original_empty_predecessor() {
    original_custody(true);
}
#[test]
fn recorded_component_checkpoint_omits_released_components_and_plain_observations() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let original = original(execution, false);
    let checkpoint = capture(execution, &original.source);
    attached_test_medium(&original.source).file_checkpoint(checkpoint.clone());
    let source = original.source.durability.cold_source().unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
        execution,
        checkpoint.bytes(),
        Some(source),
    )
    .unwrap();
    let directory = decoded.checkpoint.directory.unwrap();
    assert_eq!(directory.manifest.roots.len(), 1);
    assert!(directory.manifest.retained_empty_components.is_empty());
    assert!(!directory.manifest.roots.iter().any(|root| root
        .recorded_components
        .contains(original.before.descriptor())
        || root
            .recorded_components
            .contains(original.after.descriptor())));

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
    attached_test_medium(&original.source).file_checkpoint(next.clone());
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
        execution,
        next.bytes(),
        original.source.durability.cold_source(),
    )
    .unwrap();
    assert_eq!(
        decoded.checkpoint.directory.unwrap().manifest.roots.len(),
        1
    );
    drop(observed);
}
#[test]
fn recorded_component_checkpoint_refuses_chk15_before_source_admission() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let checkpoint = capture(execution, &runtime);
    let mut bytes = checkpoint.bytes().to_vec();
    let old = b"WORTH-REL-CHK15\n";
    bytes[..old.len()].copy_from_slice(old);
    let stop =
        crate::durability::log::native_checkpoint_codec::decode_checkpoint(execution, &bytes)
            .unwrap_err();
    assert_eq!(
        expect_durability_failure(stop).class,
        crate::durability::data::RecoveryFailureClass::UnsupportedLegacySemantics
    );
}
