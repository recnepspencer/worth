//! Real performed data and prepaid Ready capacity for registry finish tests.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::*;

pub struct Fixture {
    pub registry: WorthQueryOutputDemandRegistry,
    pub interest: WorthQueryOutputDemandInterest,
    pub upstream: WorthQueryOutputDemandInterest,
    pub dependent: WorthQueryOutputDemandInterest,
    pub receipt: WorthQueryApplicationCommitReceipt,
    pub commit: worth_runtime_world::facade::CompositeCommitIdentity,
    pub change: *const (),
    pub ready: *const (),
}

impl Fixture {
    pub fn new(reopened: bool) -> Self {
        let mut receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
        let commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
        let change_identity = Arc::as_ptr(&change).cast::<()>();
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation_for_test()
            .unwrap();
        let registry = WorthQueryOutputDemandRegistry::default();
        let scope = receipt.principal_scope().scope();
        let admit = |name, slot, predecessor| {
            registry
                .admit_for_test(
                    key(name, 1, slot),
                    None,
                    scope,
                    receipt.product_branch().occurrence_for_test(),
                    DemandAdmissionKind::Ordinary,
                    None,
                    predecessor,
                )
                .unwrap()
                .0
        };
        let mut interest = admit("interrupted-scheduling", 1, None);
        let upstream = admit("retained-upstream", 2, None);
        let dependent = admit("retained-dependent", 3, None);
        let mut ready_identity = std::ptr::null();
        {
            let mut state = (*registry.state_for_test()).lock().unwrap();
            // These are published dependency claims, as in caller_chain's
            // registry fixtures; their physical Vec backings are accounted.
            for (reader, input) in [(&interest, &upstream), (&dependent, &interest)] {
                let claims = vec![Arc::new((*input.key_for_test()).clone())];(*

                (*state.records_mut_for_test()).get_mut(&(*reader.key_for_test())).unwrap().prerequisites_mut_for_test()) = claims;(*
                (*state
                    .records_mut_for_test())
                    .get_mut(&(*input.key_for_test()))
                    .unwrap()
                    .framework_required_count_mut_for_test()) += 1;
            }
            for input in if reopened {
                vec![&upstream, &interest]
            } else {
                vec![&upstream]
            } {
                let ready = PreparedReadyBacking::new_for_test().complete(WorthQueryCompletedOutputDemand::new_for_test(WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()), WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test()));
                if (*input.key_for_test()) == (*interest.key_for_test()) {
                    ready_identity = std::ptr::from_ref(&*ready).cast::<()>();
                }(*
                (*state.records_mut_for_test()).get_mut(&(*input.key_for_test())).unwrap().state_mut_for_test()) = DemandState::Output(
                    WorthQueryOutputProgress::new_for_test(WorthQueryOutputCheckpoint::Ready(ready)),
                );
            }
        }
        if reopened {
            interest = admit(
                "interrupted-scheduling",
                1,
                Some(OutputRefreshPredecessor::Committed(&receipt)),
            );
        } else {(*(*
            (*registry
                .state_for_test())
                .lock()
                .unwrap()
                .records_mut_for_test())
                .get_mut(&(*interest.key_for_test()))
                .unwrap()
                .successor_of_mut_for_test()) = Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::succession::Succession::new_for_test(
                *receipt.idempotency_binding().key_identity(),
            ));
        }
        {
            let mut state = (*registry.state_for_test()).lock().unwrap();
            let mut row = (*state.records_mut_for_test()).get_mut(&(*interest.key_for_test())).unwrap();
            (*row.performed_source_mut_for_test()) = Some(WorthQueryPerformedOutputDemandSource::new_for_test(receipt.clone(), change, observation, None));
            (*row.performed_obligations_mut_for_test()) = vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::PerformedOutputObligation::new_for_test(commit.clone(), (* (*interest.key_for_test()).source_for_test()).clone())];

            row.finish_for_test();
        }
        Self {
            registry,
            interest,
            upstream,
            dependent,
            receipt,
            commit,
            change: change_identity,
            ready: ready_identity,
        }
    }

    pub fn next_interest(&self, reopened: bool) -> WorthQueryOutputDemandInterest {
        self.registry
            .admit_for_test(
                (*self.interest.key_for_test()).clone(),
                None,
                self.receipt.principal_scope().scope(),
                self.receipt.product_branch().occurrence_for_test(),
                DemandAdmissionKind::Ordinary,
                None,
                reopened.then_some(OutputRefreshPredecessor::Committed(&self.receipt)),
            )
            .unwrap()
            .0
    }
}
