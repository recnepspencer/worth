//! Produce only the actual original Native effect, retaining its real World open.
use super::*;

pub(super) fn publish(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    RelationalCallerAssociation,
    (u64, u64, u64),
    worth_relational::facade::history::RelationalCommitReceipt,
    worth_store_application::AdmittedApplicationHomeProgramStep,
) {
    let evaluated = Arc::new(AtomicUsize::new(0));
    let (association, identities, receipt, grant) = {
        let mut fixture = super::super::transition_opening::open_transition(location, execution);
        let contracts = original::contracts(&fixture, execution, evaluated.clone());
        let (grant, opened) = original::original_open(&mut fixture, &contracts, execution);
        let association =
            RelationalCallerAssociation::from_bytes(opened.caller_association_bytes()).unwrap();
        let ids = (
            opened.attempt_identity().ordinal(),
            opened.reserved_commit_identity().ordinal(),
            opened.unpublished_recovery_handle().identity().ordinal(),
        );
        let native = fixture.home.relational().runtime();
        let source = opened.expected_head().clone();
        let mut transaction = native
            .begin_home_program_step(
                execution,
                source.basis().relational_basis(),
                worth_relational::facade::runtime::RelationalHomeProgramStepContracts::Target {
                    contracts: &contracts,
                    transition: None,
                },
                grant.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(grant.plan().identity(), 0),
                grant.original(),
                &association,
                grant.admission(),
            )
            .unwrap();
        transaction
            .push_batch(crate::batches::create(0), execution.allocation_policy())
            .unwrap();
        let proposal = transaction.validate(execution, native).unwrap();
        let (opened, proposal) = opened.bind_query_native_proposal(proposal).unwrap();
        let candidate = native
            .prepare_validated_proposal(execution, proposal)
            .unwrap();
        let performed = match native
            .publication_port()
            .compare_and_publish(execution, candidate)
        {
            RelationalPublicationOutcome::Performed(performed) => performed,
            outcome => panic!("the actual canonical Native effect must commit: {outcome:?}"),
        };
        let result = native
            .settlement_port()
            .settle_performed_publication(performed)
            .unwrap();
        let receipt = result.semantic_result().receipt().clone();
        native
            .snapshots()
            .release_snapshot(&result.snapshot)
            .unwrap();
        drop(result);
        assert!(evaluated.load(Ordering::SeqCst) > 0);
        // The actual opened World attempt is retained, but no World occurrence
        // has been prepared or published. Native has its original durable effect.
        drop(opened.retain());
        assert_eq!(
            fixture
                .home
                .world()
                .observation_port()
                .observe_root_product_branch()
                .unwrap()
                .reference_generation()
                .get(),
            0
        );
        drop((source, contracts, fixture));
        (association, ids, receipt, grant)
    };
    (association, identities, receipt, grant)
}
