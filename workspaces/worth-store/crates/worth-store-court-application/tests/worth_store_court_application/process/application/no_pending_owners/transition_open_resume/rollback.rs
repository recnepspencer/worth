//! A denied reactivation restores the resident original open and its capacity.
use super::*;
use worth_runtime_world::facade::{
    NoEffectCause, RuntimeWorldRecoveryRecordState, WorldHomeTransitionOpenResumeDenial,
};

#[test]
fn a_transition_open_resource_refusal_restores_the_same_recovered_custody_for_retry() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let evaluated = Arc::new(AtomicUsize::new(0));
    let identities = {
        let mut fixture =
            super::super::transition_opening::open_transition(directory.path(), &execution);
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        let (grant, opened) = original::original_open(&mut fixture, &contracts, &execution);
        let ids = (
            opened.attempt_identity().ordinal(),
            opened.reserved_commit_identity().ordinal(),
            opened.unpublished_recovery_handle().identity().ordinal(),
        );
        drop(opened.retain());
        drop((grant, contracts, fixture));
        ids
    };
    {
        // The root fits. Continuing the original open needs a second history
        // seat; this real owner refuses it after selecting the recovery slot.
        let fixture =
            worth_store_testkit_application_oracle::world_home::open_with_retained_history_limit(
                super::super::transition_opening::successor_request(directory.path()),
                &execution,
                1,
            )
            .expect("the actual original root and cold open fit the installed policy");
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        let page = fixture
            .home
            .world()
            .inspection_port()
            .recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(page.rows().len(), 1);
        let claim = fixture
            .home
            .world()
            .recovery_port()
            .inspect_recovered_effects(page.rows()[0].handle())
            .unwrap();
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
        let refused = fixture
            .home
            .world()
            .publication_port()
            .resume_home_transition_open(
                &execution,
                admitted.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(admitted.plan().identity(), 0),
                admitted.original(),
                admitted.admission(),
                claim,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                &cancellation.token(),
                None,
            )
            .expect_err("the real history owner must refuse the second seat");
        assert!(matches!(
            refused.reason(),
            WorldHomeTransitionOpenResumeDenial::Source(NoEffectCause::CapacityExhausted)
        ));
        let (_, claim) = refused.into_parts();
        assert_eq!(claim.attempt_identity().ordinal(), identities.0);
        assert_eq!(claim.reserved_commit_identity().ordinal(), identities.1);
        assert_eq!(claim.identity().ordinal(), identities.2);
        assert!(claim.native_receipt().is_none());
        let inspection = fixture.home.world().inspection_port();
        let snapshot = inspection.recovery_snapshot().unwrap();
        assert_eq!(snapshot.reserved(), 0);
        assert_eq!(snapshot.installed(), 1);
        let after = inspection
            .recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(after.rows().len(), 1);
        assert_eq!(
            after.rows()[0].state(),
            RuntimeWorldRecoveryRecordState::Recovered
        );
        assert_eq!(after.rows()[0].handle().identity().ordinal(), identities.2);
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
        drop((claim, page, after, admitted, contracts));
        close(fixture.home, &execution);
    }
    let fixture = original::reopen(directory.path(), &execution);
    let contracts = original::contracts(&fixture, &execution, evaluated.clone());
    let page = fixture
        .home
        .world()
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let claim = fixture
        .home
        .world()
        .recovery_port()
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
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
    let opened = fixture
        .home
        .world()
        .publication_port()
        .resume_home_transition_open(
            &execution,
            admitted.plan(),
            HomeProgramTransitionStepAssociation::from_recorded(admitted.plan().identity(), 0),
            admitted.original(),
            admitted.admission(),
            claim,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            &cancellation.token(),
            None,
        )
        .unwrap();
    assert_eq!(opened.attempt_identity().ordinal(), identities.0);
    assert_eq!(opened.reserved_commit_identity().ordinal(), identities.1);
    assert_eq!(
        opened.unpublished_recovery_handle().identity().ordinal(),
        identities.2
    );
    assert_eq!(evaluated.load(Ordering::SeqCst), 0);
    // This control stops before Native. Keep the actual durable open in World
    // through explicit retention, then abandon the pending Home honestly.
    drop(opened.retain());
    drop((admitted, contracts, page, fixture));
}
