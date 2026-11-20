//! The real cell owns consuming movement refusal and post-commit unwind.
use super::progression::CurrentOriginal;
use super::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use worth_runtime_world::facade::{
    RuntimeWorldCloseDenial, RuntimeWorldOwnerLifecycleObservation, RuntimeWorldServiceDenial,
    WorldHomeTransitionEffectInterruption as Mode, WorldHomeTransitionEffectProgress as Progress,
};

#[test]
fn consuming_original_world_movement_refusal_preserves_custody_for_actual_home_reopen() {
    interrupted(Mode::RefuseHeadMovement);
}

#[test]
fn unwind_at_the_actual_committed_world_head_keeps_the_original_in_reopen_required() {
    interrupted(Mode::UnwindAfterHeadMovement);
}

#[test]
fn dropping_an_original_interruption_guard_disarms_only_its_receiver_control() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (_, _, receipt, _earlier) = native_original::publish(directory.path(), &execution);
    let mut current = CurrentOriginal::open(directory.path(), &execution);
    let port = current.fixture.home.world().publication_port();
    let control = port
        .arm_home_transition_effect_control(
            &execution,
            current.claim.as_ref().unwrap(),
            Mode::RefuseHeadMovement,
        )
        .unwrap();
    assert!(!control.was_triggered());
    assert!(matches!(
        port.arm_home_transition_effect_control(
            &execution,
            current.claim.as_ref().unwrap(),
            Mode::UnwindAfterHeadMovement
        ),
        Err(
            worth_runtime_world::facade::WorldHomeTransitionEffectDenial::Recovery(
                RuntimeWorldRecoveryDenial::Busy
            )
        )
    ));
    drop(control);
    let replacement = port
        .arm_home_transition_effect_control(
            &execution,
            current.claim.as_ref().unwrap(),
            Mode::UnwindAfterHeadMovement,
        )
        .unwrap();
    drop(replacement);
    let publication = current.finish(&execution);
    progression::assert_live_head(&current.fixture, &publication);
    assert_eq!(publication.native_receipt(), &receipt);
    assert_eq!(current.executions.load(Ordering::SeqCst), 0);
    drop((
        publication,
        current.contracts,
        current.grant,
        current.claim,
        port,
    ));
    close(current.fixture.home, &execution);
}

fn interrupted(mode: Mode) {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (association, identities, receipt, _earlier) =
        native_original::publish(directory.path(), &execution);
    let (control, executions) = {
        let mut current = CurrentOriginal::open(directory.path(), &execution);
        let control = current
            .fixture
            .home
            .world()
            .publication_port()
            .arm_home_transition_effect_control(&execution, current.claim.as_ref().unwrap(), mode)
            .unwrap();
        current.until_acknowledged(&execution);
        assert!(!control.was_triggered());
        match mode {
            Mode::RefuseHeadMovement => {
                current.claim = Some(match current.advance(&execution) {
                    Outcome::ReopenRequired(claim) => claim,
                    other @ (Outcome::Continued(_)
                    | Outcome::Pending(_)
                    | Outcome::EvidenceRequired(_)
                    | Outcome::Indeterminate(_)
                    | Outcome::Refused { .. }) => {
                        panic!("spent movement requires reopen: {other:?}")
                    }
                });
            }
            Mode::UnwindAfterHeadMovement => {
                assert!(catch_unwind(AssertUnwindSafe(|| current.advance(&execution))).is_err());
                assert!(
                    current.claim.is_none(),
                    "unwind dropped the consuming caller claim"
                );
                current.claim = Some(
                    current
                        .fixture
                        .home
                        .world()
                        .recovery_port()
                        .inspect_recovered_effects(&current.handle)
                        .unwrap(),
                );
            }
        }
        assert!(control.was_triggered());
        let actual = current
            .fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap();
        match mode {
            Mode::RefuseHeadMovement => assert_eq!(actual.reference_generation().get(), 0),
            Mode::UnwindAfterHeadMovement => {
                assert_eq!(actual.reference_generation().get(), 1);
                assert_eq!(actual.selected_commit().ordinal(), identities.1);
                assert_eq!(
                    actual.publication_attempt().unwrap().ordinal(),
                    identities.0
                );
            }
        }
        drop(actual);
        assert_eq!(
            current
                .claim
                .as_ref()
                .unwrap()
                .home_transition_effect_progress()
                .unwrap(),
            Some(Progress::ReopenRequired)
        );
        assert_eq!(
            current.claim.as_ref().unwrap().native_receipt().as_ref(),
            Some(&receipt)
        );
        current.refuse_release_and_reclaim(&execution);
        current.claim = Some(match current.advance(&execution) {
            Outcome::ReopenRequired(claim) => claim,
            other @ (Outcome::Continued(_)
            | Outcome::Pending(_)
            | Outcome::EvidenceRequired(_)
            | Outcome::Indeterminate(_)
            | Outcome::Refused { .. }) => {
                panic!("the original spent permit cannot be reset: {other:?}")
            }
        });
        assert_eq!(current.executions.load(Ordering::SeqCst), 0);
        // Remove the caller claim before close: the genuine retained recovery
        // operation, not a claimed-handle guard, must keep the World open.
        drop(current.claim.take());
        let CurrentOriginal {
            fixture,
            grant,
            contracts,
            claim,
            executions,
            ..
        } = current;
        drop((grant, contracts, claim));
        let refused = match fixture.home.seal_world(&execution) {
            Ok(_) => panic!("the original continuation must refuse actual Home close"),
            Err(refused) => refused,
        };
        assert!(matches!(
            refused.reason(),
            worth_store_application::ApplicationHomeCloseDenial::World(
                RuntimeWorldServiceDenial::Denied(RuntimeWorldCloseDenial::InFlightCriticalSection)
            )
        ));
        let home = refused.resume().unwrap_or_else(|refused| {
            panic!(
                "refused close preserves its serving owner: {:?}",
                refused.reason()
            )
        });
        assert_eq!(
            home.world().lifecycle_observation(),
            RuntimeWorldOwnerLifecycleObservation::Open
        );
        // Honest abandonment of the real owner retains journal truth. The
        // still-live control below is port-free and cannot hold the old lease.
        drop(home);
        (control, executions)
    };
    let fixture = original::reopen(directory.path(), &execution);
    let actual = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(actual.reference_generation().get(), 1);
    assert_eq!(actual.selected_commit().ordinal(), identities.1);
    assert_eq!(
        actual.publication_attempt().unwrap().ordinal(),
        identities.0
    );
    let subjects = fixture
        .home
        .relational()
        .runtime()
        .durability_authority()
        .discover_associated_effects(&execution, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].receipt(), &receipt);
    assert_eq!(
        fixture
            .home
            .world()
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    assert_eq!(executions.load(Ordering::SeqCst), 0);
    assert!(
        control.was_triggered(),
        "the old observer survives without retaining live Home resources"
    );
    drop((actual, subjects));
    close(fixture.home, &execution);
    drop(control);
}
