//! Real cold checkpoints preserve both original component bases of a publication.
//! The C2 portions of
//! 0615dd733c:crates/worth-relational/src/tests/durability/contracts/recorded_component_checkpoint.rs
//! (`original_custody`) are restored in place: the selected and unrelated
//! envelope cells stay cold, the selected envelope demand carries the readmitted
//! semantic result, and both original roots remain fully demandable. Fixture
//! adaptations: residency comes from the owner's `observe_commit_envelope_custody`;
//! the envelope demand is the public replay read of the same cell; full root
//! demand and its snapshot are `RelationalBasisCustodyWitness::read_retained_full`.
//! The unrelated-artifact deletion belongs to C4 and is not restored here.
//! `recorded_original_retirement_preserves_both_leases_across_busy_and_stop`
//! restores the original `retire` branch and its same-SHA fixture
//! `assert_recovered_cut_and_lane_busy` (0615dd733c retained_release/tests.rs)
//! through the owner's test doors (C2 ruling 2026-11-21): the fixture's held
//! external cut and lane lock are `advance_with_retention_gate_held_for_test`,
//! its private lane counters are `retention_census_for_test`, and the private
//! terminal accounting is `external_retention_terminal_counts_for_test`.
use super::*;
use worth_relational::facade::{
    change_source::RelationalRetainedObservationRetirementProgress as Progress,
    durability::{
        RelationalRecoveredPublicationRetirement, RelationalRetentionGateForTest as Gate,
    },
    history::CommitId,
    runtime::RelationalRuntime,
};

fn resident(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    commit: CommitId,
) -> bool {
    runtime
        .observe_commit_envelope_custody()
        .into_iter()
        .find(|cell| cell.commit_id == commit)
        .expect("retained commit cell")
        .resident
}

fn readmit_original(empty: bool, retire: bool) {
    let stop = worth_execution::CancellationSource::new();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request()
        .with_cancellation(stop.token());
    let request = &ExecutionRequest::serial(&host);
    let original = original(request, empty);
    let offers = original.medium.offered().len();
    let (runtime, authority) = recover(request, &original.medium);
    let subject = runtime
        .durability_authority()
        .discover_associated_effects(request, &original.association)
        .unwrap()
        .pop()
        .unwrap();
    assert!(!resident(&runtime, subject.receipt().commit_id));
    let effect = runtime
        .durability_authority()
        .readmit_associated_effect(request, &subject)
        .unwrap();
    let frame = Artifact::readmit(
        &original.medium.artifact_bytes(subject.position()).unwrap(),
        request,
    )
    .unwrap();
    let semantic = frame.envelope().semantic_result();
    assert_eq!(effect.semantic_result(), semantic);
    let selected = runtime
        .replay()
        .canonical_commit_envelope(request, subject.receipt().commit_id)
        .unwrap()
        .unwrap();
    assert_eq!(effect.semantic_result(), selected.semantic_result());
    let proof = authority
        .admit_recorded_publication(request, &runtime, effect, &original.before, &original.after)
        .unwrap();
    assert_eq!(proof.custody().effect().semantic_result(), semantic);
    let shared: &RelationalRuntime = &runtime;
    shared.run_branch_root_reclamation_pass();
    let before_count = if empty { 0 } else { 1 };
    for (basis, recorded, count) in [
        (
            proof.custody().predecessor(),
            &original.before,
            before_count,
        ),
        (
            proof.custody().successor(),
            &original.after,
            before_count + 1,
        ),
    ] {
        let exported = runtime
            .owner_component_services()
            .basis_port()
            .export_recorded_retained_component(request, basis)
            .unwrap();
        assert_eq!(&exported, recorded);
        let witness = basis.custody_witness();
        assert_eq!(
            witness.read_retained_full(request, &runtime).unwrap(),
            Some(witness.schema_authority_digest())
        );
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let read = runtime
            .read_truth()
            .read_snapshot(&snapshot, request)
            .expect("original retained read remains funded")
            .expect("original retained snapshot remains readable");
        assert_eq!(
            read.entities().len(),
            count,
            "the actual original basis remains readable"
        );
        assert!(read.relations().is_empty());
        drop(read);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    }
    assert!(runtime
        .owner_component_services()
        .basis_port()
        .compare_current_exact(proof.custody().successor())
        .is_err());
    assert_eq!(
        original.medium.offered().len(),
        offers,
        "recovery never republishes"
    );
    assert!(!resident(&runtime, original.unrelated_id));
    if retire {
        let basis = proof.custody().predecessor().clone();
        let before = runtime.external_retention_terminal_counts_for_test();
        let mut task = proof.consume(|original| original.into_retirement());
        assert_recovered_cut_and_lane_busy(&mut task, &basis, request);
        drop(basis);
        stop.cancel();
        assert!(matches!(task.advance(request), Progress::Stopped(_)));
        let terminal = runtime.external_retention_terminal_counts_for_test();
        assert_eq!(terminal.explicit_releases, before.explicit_releases + 1);
        assert_eq!(terminal.dropped_releases, before.dropped_releases);
        let mut task = task.into_retired().unwrap_err();
        let retry = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let retry = ExecutionRequest::serial(&retry);
        let retired = loop {
            match task.advance(&retry) {
                Progress::Advanced => continue,
                Progress::Complete => break task.into_retired().unwrap(),
                other => panic!("original recovered custody retirement: {other:?}"),
            }
        };
        let terminal = runtime.external_retention_terminal_counts_for_test();
        assert_eq!(terminal.explicit_releases, before.explicit_releases + 2);
        assert_eq!(terminal.dropped_releases, before.dropped_releases);
        runtime.prepare_close().unwrap().finish();
        retired.reclaim();
    } else {
        drop(proof);
        runtime.prepare_close().unwrap().finish();
    }
}

/// The aggregate must preserve its two original lease tasks on either gate.
fn assert_recovered_cut_and_lane_busy(
    task: &mut RelationalRecoveredPublicationRetirement,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    request: &ExecutionRequest<'_, '_>,
) {
    let cut = task
        .advance_with_retention_gate_held_for_test(Gate::ExternalCut, basis, request)
        .unwrap();
    assert_eq!(cut.progress, Progress::Busy);
    assert_eq!(cut.after.counters, cut.before.counters);
    let lane = task
        .advance_with_retention_gate_held_for_test(Gate::Lane, basis, request)
        .unwrap();
    assert_eq!(lane.progress, Progress::Busy);
    assert!(lane.before.root_obligations.is_some());
    assert_eq!(lane.after.lane_obligations, lane.before.lane_obligations);
    assert_eq!(lane.after.root_obligations, lane.before.root_obligations);
    assert_eq!(task.advance(request), Progress::Advanced);
    assert_eq!(
        basis
            .retention_census_for_test()
            .counters
            .external_pin_releases,
        cut.before.counters.external_pin_releases + 1
    );
}

#[test]
fn recorded_component_checkpoint_readmits_cold_canonical_predecessor_and_successor() {
    readmit_original(false, false);
}

#[test]
fn recorded_component_checkpoint_readmits_original_empty_predecessor() {
    readmit_original(true, false);
}

#[test]
fn recorded_original_retirement_preserves_both_leases_across_busy_and_stop() {
    readmit_original(false, true);
}
