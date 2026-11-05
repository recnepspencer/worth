//! An admitted original open resumes on the same directory and original identities.
use super::transition_open_original as original;
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::{
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
    HomeProgramTransitionTargetMeaning,
};
use worth_relational::facade::{
    mvcc::RelationalCallerAssociation, runtime::RelationalHomeProgramContractDenial,
};
use worth_runtime_world::facade::RuntimeWorldCancellationSource;
mod rollback;

#[test]
fn a_readmitted_transition_open_keeps_its_original_attempt_commit_and_custody() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let before = Arc::new(AtomicUsize::new(0));
    let (earlier, association, identities, incarnation) = {
        let mut fixture = super::transition_opening::open_transition(directory.path(), &execution);
        let contracts = original::contracts(&fixture, &execution, before.clone());
        let (grant, opened) = original::original_open(&mut fixture, &contracts, &execution);
        let ids = (
            opened.attempt_identity().ordinal(),
            opened.reserved_commit_identity().ordinal(),
            opened.unpublished_recovery_handle().identity().ordinal(),
        );
        let association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let incarnation = fixture.home.incarnation();
        drop(opened.retain());
        drop(contracts);
        drop(fixture);
        (grant, association, ids, incarnation)
    };
    assert_eq!(
        before.load(Ordering::SeqCst),
        0,
        "the original stopped before any Native target effect"
    );
    let evaluated = Arc::new(AtomicUsize::new(0));
    let receipt = {
        let fixture = original::reopen(directory.path(), &execution);
        assert_ne!(fixture.home.incarnation(), incarnation);
        let contracts = original::contracts(&fixture, &execution, evaluated.clone());
        let target =
            HomeProgramTransitionTargetMeaning::from_recorded(0, contracts.catalog_digest());
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
        assert_eq!(claim.attempt_identity().ordinal(), identities.0);
        assert_eq!(claim.reserved_commit_identity().ordinal(), identities.1);
        assert_eq!(claim.identity().ordinal(), identities.2);
        assert!(claim.native_receipt().is_none());
        assert!(matches!(
            fixture.home.admit_readmitted_home_program_transition_step(
                0,
                &claim,
                HomeProgramTransitionPlanDigest::from_recorded([8; 32]),
                1,
                target,
                &execution,
            ),
            Err(
                worth_store_application::ApplicationHomeTransitionIntentDenial::TransitionPlanDrift
            )
        ));
        let admitted = fixture
            .home
            .admit_readmitted_home_program_transition_step(
                0,
                &claim,
                HomeProgramTransitionPlanDigest::from_recorded([7; 32]),
                1,
                target,
                &execution,
            )
            .unwrap();
        let expected = claim.expected_head().clone();
        let native = fixture.home.relational().runtime();
        assert!(matches!(
            native.begin_home_program_step(
                &execution,
                expected.basis().relational_basis(),
                worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: None
                },
                earlier.plan(),
                worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                    earlier.plan().identity(),
                    earlier.plan().target_meaning().step()
                ),
                earlier.original(),
                &association,
                earlier.admission(),
            ),
            Err(RelationalHomeProgramContractDenial::Admission(
                worth_proof::HomeProgramTransitionAdmissionDenial::StaleIncarnation
            ))
        ));
        assert_eq!(evaluated.load(Ordering::SeqCst), 0);
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
        assert_eq!(opened.caller_association_bytes(), association.as_bytes());
        assert_eq!(
            evaluated.load(Ordering::SeqCst),
            0,
            "readmission and reactivation never run the target"
        );
        let mut transaction = native
            .begin_home_program_step(
                &execution,
                expected.basis().relational_basis(),
                worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: None,
                },
                admitted.plan(),
                worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                    admitted.plan().identity(),
                    admitted.plan().target_meaning().step(),
                ),
                admitted.original(),
                &association,
                admitted.admission(),
            )
            .unwrap();
        transaction
            .push_batch(crate::batches::create(0), execution.allocation_policy())
            .unwrap();
        let proposal = transaction.validate(&execution, native).unwrap();
        let prepared = opened.prepare_native(native, &execution, proposal).unwrap();
        let performed = fixture.publish(&execution, prepared, &cancellation.token());
        assert_eq!(performed.attempt_identity().ordinal(), identities.0);
        let receipt = performed
            .component_results()
            .relational_commit_result()
            .unwrap()
            .semantic_result()
            .receipt()
            .clone();
        drop(performed.consume());
        assert!(evaluated.load(Ordering::SeqCst) > 0);
        let subjects = native
            .durability_authority()
            .discover_associated_effects(&execution, &association)
            .unwrap();
        assert_eq!(subjects.len(), 1);
        assert_eq!(subjects[0].receipt(), &receipt);
        drop((subjects, expected, page, admitted, contracts));
        close(fixture.home, &execution);
        receipt
    };
    let fixture = original::reopen(directory.path(), &execution);
    let selected = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    assert_eq!(selected.reference_generation().get(), 1);
    assert_eq!(
        selected.publication_attempt().unwrap().ordinal(),
        identities.0
    );
    assert_eq!(selected.selected_commit().ordinal(), identities.1);
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
}
