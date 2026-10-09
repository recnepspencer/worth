//! Real performed data and prepaid Ready capacity for registry finish tests.
use super::*;

pub(super) struct Fixture {
    pub(super) registry: WorthQueryOutputDemandRegistry,
    pub(super) interest: WorthQueryOutputDemandInterest,
    pub(super) upstream: WorthQueryOutputDemandInterest,
    pub(super) dependent: WorthQueryOutputDemandInterest,
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) commit: worth_runtime_world::facade::CompositeCommitIdentity,
    pub(super) change: *const (),
    pub(super) ready: *const (),
}

impl Fixture {
    pub(super) fn new(reopened: bool) -> Self {
        let mut receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
        let commit = receipt
            .committed_product_publication()
            .composite_commit()
            .clone();
        let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
        let change_identity = Arc::as_ptr(&change).cast::<()>();
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation()
            .unwrap();
        let registry = WorthQueryOutputDemandRegistry::default();
        let scope = receipt.principal_scope().scope();
        let admit = |name, slot, predecessor| {
            registry
                .admit(
                    key(name, 1, slot),
                    None,
                    scope,
                    receipt.product_branch().occurrence(),
                    DemandAdmissionKind::Ordinary,
                    None,
                    predecessor,
                    &mut record_admission(),
                )
                .unwrap()
        };
        let mut interest = admit("interrupted-scheduling", 1, None);
        let upstream = admit("retained-upstream", 2, None);
        let dependent = admit("retained-dependent", 3, None);
        let mut ready_identity = std::ptr::null();
        {
            let mut state = registry.state.lock().unwrap();
            // These are published dependency claims, as in caller_chain's
            // registry fixtures; their physical Vec backings are accounted.
            for (reader, input) in [(&interest, &upstream), (&dependent, &interest)] {
                let claims = vec![Arc::new(input.key.clone())];
                state.required_reserved_bytes +=
                    claims.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
                state.records.get_mut(&reader.key).unwrap().prerequisites = claims;
                state
                    .records
                    .get_mut(&input.key)
                    .unwrap()
                    .framework_required_count += 1;
            }
            for input in if reopened {
                vec![&upstream, &interest]
            } else {
                vec![&upstream]
            } {
                let ready = PreparedReadyBacking::prepare(&state, &mut record_admission(), 0)
                    .unwrap()
                    .complete(WorthQueryCompletedOutputDemand {
                        authority: WorthQueryAcceptedOutputAuthority::Committed(receipt.clone()),
                        readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),
                        resources: None,
                    });
                if input.key == interest.key {
                    ready_identity = std::ptr::from_ref(&*ready).cast::<()>();
                }
                state.records.get_mut(&input.key).unwrap().state = DemandState::Output(
                    WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Ready(ready)),
                );
            }
        }
        if reopened {
            interest = admit(
                "interrupted-scheduling",
                1,
                Some(OutputRefreshPredecessor::Committed(&receipt)),
            );
        } else {
            registry
                .state
                .lock()
                .unwrap()
                .records
                .get_mut(&interest.key)
                .unwrap()
                .successor_of = Some(super::super::super::succession::Succession::new(
                *receipt.idempotency_binding().key_identity(),
            ));
        }
        {
            let mut state = registry.state.lock().unwrap();
            let row = state.records.get_mut(&interest.key).unwrap();
            row.performed_source = Some(WorthQueryPerformedOutputDemandSource {
                receipt: receipt.clone(),
                change,
                observation,
                output_source_identity: None,
            });
            row.performed_obligations = vec![super::super::super::PerformedOutputObligation {
                source_commit: commit.clone(),
                source: interest.key.source.clone(),
            }];
            state.obligation_reserved_bytes += row.obligation_reserved_bytes();
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

    pub(super) fn custody(&self) -> (usize, usize, usize, usize) {
        let state = self.registry.state.lock().unwrap();
        (
            state.obligation_reserved_bytes,
            state.required_reserved_bytes,
            state
                .required_custody_retained_bytes
                .load(Ordering::Acquire),
            state.record_retained_bytes.load(Ordering::Acquire),
        )
    }

    pub(super) fn next_interest(&self, reopened: bool) -> WorthQueryOutputDemandInterest {
        self.registry
            .admit(
                self.interest.key.clone(),
                None,
                self.receipt.principal_scope().scope(),
                self.receipt.product_branch().occurrence(),
                DemandAdmissionKind::Ordinary,
                None,
                reopened.then_some(OutputRefreshPredecessor::Committed(&self.receipt)),
                &mut record_admission(),
            )
            .unwrap()
    }
}
