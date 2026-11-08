//! The real Native-only interruption resumes its original World occurrence.
use super::{transition_open_original as original, *};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
};
use worth_relational::facade::mvcc::{RelationalCallerAssociation, RelationalPublicationOutcome};
use worth_runtime_world::facade::{
    RuntimeWorldCancellationSource, RuntimeWorldRecoveryDenial,
    WorldHomeTransitionEffectOutcome as Outcome, WorldRecoveredReleaseOutcome,
};

mod affinity;
mod allocation;
mod interruption;
mod native_original;
mod progression;

#[test]
fn a_durable_native_transition_effect_continues_its_original_world_occurrence_without_execution() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let (association, identities, receipt, _earlier) =
        native_original::publish(directory.path(), &execution);
    let resumed = Arc::new(AtomicUsize::new(0));
    {
        let fixture = original::reopen(directory.path(), &execution);
        let contracts = original::contracts(&fixture, &execution, resumed.clone());
        let page = fixture
            .home
            .world()
            .inspection_port()
            .recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let recovery = fixture.home.world().recovery_port();
        let claim = recovery
            .inspect_recovered_effects(page.rows()[0].handle())
            .unwrap();
        assert_eq!(claim.attempt_identity().ordinal(), identities.0);
        assert_eq!(claim.reserved_commit_identity().ordinal(), identities.1);
        assert_eq!(claim.identity().ordinal(), identities.2);
        assert_eq!(claim.native_receipt().as_ref(), Some(&receipt));
        // Generic disposal must not write a release for the transition's Native effect.
        let claim = match recovery.release_recovered_effects(&execution, claim) {
            WorldRecoveredReleaseOutcome::AdmissionRefused {
                cause: RuntimeWorldRecoveryDenial::HomeProgramTransitionRequired,
                custody,
            } => custody,
            other => panic!("the transition owner must retain its original effect: {other:?}"),
        };
        let admitted = fixture
            .home
            .admit_readmitted_home_program_transition_step(
                0,
                &claim,
                HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
                1,
                contracts.target_meaning(&execution, 0).unwrap(),
                &crate::batches::transition_body_roster(&execution, 1),
                &execution,
            )
            .unwrap();
        let cancellation = RuntimeWorldCancellationSource::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let port = fixture.home.world().publication_port();
        let mut claim = claim;
        let publication = loop {
            assert!(
                std::time::Instant::now() < deadline,
                "the original occurrence must finish"
            );
            match port.continue_home_transition_effect(
                &execution,
                admitted.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(admitted.plan().identity(), 0),
                admitted.original(),
                admitted.admission(),
                claim,
                &cancellation.token(),
                None,
            ) {
                Outcome::Continued(publication) => break publication,
                Outcome::Pending(pending) => {
                    // The same actual continuation survives dropping/reclaiming a
                    // claim and cannot race an independent release journal.
                    claim = match recovery.release_recovered_effects(&execution, pending) {
                        WorldRecoveredReleaseOutcome::AdmissionRefused {
                            cause: RuntimeWorldRecoveryDenial::Busy,
                            custody,
                        } => custody,
                        other => panic!("live continuation refuses disposition: {other:?}"),
                    };
                    drop(claim);
                    claim = recovery
                        .inspect_recovered_effects(page.rows()[0].handle())
                        .unwrap();
                    assert_eq!(claim.identity().ordinal(), identities.2);
                    std::thread::yield_now();
                }
                other => panic!("the same original World occurrence must continue: {other:?}"),
            }
        };
        assert_eq!(publication.attempt_identity().ordinal(), identities.0);
        assert_eq!(
            publication.after().selected_commit().ordinal(),
            identities.1
        );
        assert_eq!(publication.native_receipt(), &receipt);
        progression::assert_live_head(&fixture, &publication);
        assert_eq!(publication.before().reference_generation().get(), 0);
        assert_eq!(publication.after().reference_generation().get(), 1);
        assert_eq!(
            resumed.load(Ordering::SeqCst),
            0,
            "no target/provider executes during continuation"
        );
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
        let subjects = fixture
            .home
            .relational()
            .runtime()
            .durability_authority()
            .discover_associated_effects(&execution, &association)
            .unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].receipt(), &receipt);
        drop((subjects, publication, admitted, contracts, page));
        close(fixture.home, &execution);
    }
    let fixture = original::reopen(directory.path(), &execution);
    let selected = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(selected.reference_generation().get(), 1);
    assert_eq!(selected.selected_commit().ordinal(), identities.1);
    assert_eq!(
        selected.publication_attempt().unwrap().ordinal(),
        identities.0
    );
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
    let subjects = fixture
        .home
        .relational()
        .runtime()
        .durability_authority()
        .discover_associated_effects(&execution, &association)
        .unwrap();
    assert_eq!(subjects.len(), 1);
    assert_eq!(subjects[0].receipt(), &receipt);
    assert_eq!(resumed.load(Ordering::SeqCst), 0);
    drop((subjects, selected));
    close(fixture.home, &execution);
}
