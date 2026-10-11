//! Pending-edge resolution does not replace the cutoff's native verification.
use super::*;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use std::sync::{Mutex, OnceLock};

use crate::domain_computation::primary_graph::{
    application_attempt::{CompletedHandlerFactBoundary, Movement, PreparedDecisionReuseContext},
    application_contribution::{
        InstalledProducerEdition, PriorAbsence, WorthQueryDecisionContextDependencies,
        WorthQueryProducerInputReuseContract,
    },
    application_query::WorthQueryObservedSourceSelection,
    output_lineage::{
        input_cutoff::{
            InputCutoffDecision, InputCutoffVerificationStop, PreparedInputCutoffBasis,
            RetainedInputCutoffCandidate,
        },
        own_write_fixture::with_generated_own_write,
        retained_computation::RecordedComputation,
        ComputationSourceEvidence, PreparedInputReuseKey, RecordedOutput, RecordedOutputMutable,
    },
    DecisionContextUse,
};

enum Successor {
    Equal,
    Unequal,
    Dirty,
}

#[test]
fn equal_clean_successor_passes_the_cutoff_and_verifies_the_full_handler_prefix() {
    cutoff_journey(Successor::Equal);
}

#[test]
fn clean_unequal_or_dirty_successor_still_defers_at_the_cutoff() {
    for successor in [Successor::Unequal, Successor::Dirty] {
        cutoff_journey(successor);
    }
}

fn cutoff_journey(successor: Successor) {
    with_generated_own_write(|world, _, performed, _| {
        let graph = world.application.runtime.primary_graph().unwrap();
        let handle = graph.integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        let record = performed.get().unwrap();
        let entity = world
            .selected_product()
            .resolve_entity(
                AccountStatus::reference(),
                "2".to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
            .entity_id();
        let status = AccountStatus::reference();
        let status = graph
            .layout()
            .field_locator(status.entity(), status.aspect(), status.field())
            .unwrap()
            .clone();
        let label = AccountLabel::reference();
        let label = graph
            .layout()
            .field_locator(label.entity(), label.aspect(), label.field())
            .unwrap()
            .clone();
        let mut source = record.settlement_identity.source().clone();
        source.output_binding = OutputBindingIdentity::declared("Upstream");
        let a0 =
            RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 0);
        let a1 =
            RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 1);
        source.output_binding = OutputBindingIdentity::declared("Consumer");
        let consumer =
            RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 0);
        handle.with_runtime_mut(|runtime| {
            let (before_handle, before) = snapshot(runtime);
            let prefix: Arc<[_]> = Arc::from([field_fact(runtime, &before_handle, entity, label)]);
            register(owner, Arc::clone(&a0), Arc::from([field_fact(runtime, &before_handle, entity, status.clone())]), &before, OrdSet::new());
            // b has an authentic equality chain. Both b rows become pending
            // when a changes; a's later equality clears them one level only,
            // leaving c's pending edge to b's predecessor in place.
            source.output_binding = OutputBindingIdentity::declared("DownstreamOutput");
            let b = RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 0);
            let b_successor = RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 1);
            register(owner, Arc::clone(&b), Arc::from([]), &before, OrdSet::unit(Arc::clone(&a0)));
            register_dependent_alias(owner, &before, &b, &b_successor, &a0);
            register(owner, Arc::clone(&consumer), Arc::clone(&prefix), &before, OrdSet::unit(Arc::clone(&b)));
            write_field(runtime, entity, status.clone(), "cutoff-upstream-changed");
            let (selected_handle, selected) = snapshot(runtime);
            match successor {
                Successor::Equal | Successor::Dirty => register_alias(owner, runtime, &selected_handle, &selected, entity, status.clone(), &a0, &a1),
                Successor::Unequal => register(owner, Arc::clone(&a1), Arc::from([field_fact(runtime, &selected_handle, entity, status.clone())]), &selected, OrdSet::new()),
            }
            if matches!(successor, Successor::Dirty) {
                write_field(runtime, entity, status, "cutoff-successor-dirty");
            }
            let (live_handle, live) = snapshot(runtime);
            match successor {
                Successor::Equal | Successor::Unequal => assert!(matches!(
                    currentness(owner, &live, &a1), SourceSettlementCurrentness::Clean,
                )),
                Successor::Dirty => assert!(matches!(
                    currentness(owner, &live, &a1), SourceSettlementCurrentness::Dirty(_),
                )),
            }
            assert_pending(owner, &live, &consumer, &b);
            if matches!(successor, Successor::Equal) {
                assert!(matches!(owner.consumed_output_currentness(
                    &live, &b, &mut owner.edit_admission(),
                ).unwrap(),
                    crate::domain_computation::primary_graph::output_lineage::invalidation::ConsumedOutputCurrentness::CanonicallyEqualClean(identity)
                        if identity == b_successor),
                    "the cutoff's exact pending edge must have a certified equal clean successor");
            }
            if !matches!(successor, Successor::Equal) {
                // The cutoff must ask the Dirty predecessor directly: a clean
                // unrelated successor is not a consequence for this identity.
                register(owner, Arc::clone(&consumer), Arc::clone(&prefix), &live, OrdSet::unit(Arc::clone(&a0)));
                assert_pending(owner, &live, &consumer, &a0);
                let upstream = owner.consumed_output_currentness(
                    &live, &a0, &mut owner.edit_admission(),
                ).unwrap();
                use crate::domain_computation::primary_graph::output_lineage::invalidation::ConsumedOutputCurrentness;
                match successor {
                    Successor::Unequal => assert!(matches!(upstream,
                        ConsumedOutputCurrentness::Direct(SourceSettlementCurrentness::Dirty(_))),
                        "no equality link may connect the clean unequal successor"),
                    Successor::Dirty => assert!(matches!(upstream,
                        ConsumedOutputCurrentness::PendingEqualSuccessor)),
                    Successor::Equal => unreachable!("the equal case has its certified consequence"),
                }
            }
            let candidate = candidate(record, Arc::clone(&consumer), Arc::clone(&prefix));
            let context = context();
            let mut admission = owner.edit_admission();
            let basis = PreparedInputCutoffBasis::prepare(runtime, &live_handle, &mut admission).unwrap();
            let mut currentness = owner.edit_admission();
            let available = currentness.remaining_work();
            let result = candidate.clone().verify_for_reuse(
                candidate.prepared_input_key().unwrap().continued_by_republication(), context, None,
                runtime, &basis, owner, &mut admission, &mut currentness,
            );
            if matches!(successor, Successor::Equal) {
                assert!(matches!(result, Ok(InputCutoffDecision::Reuse(_))),
                    "the resolved pending edge must reach full verification");
                let mut expected = owner.edit_admission();
                let start = expected.remaining_work();
                assert!(ConsumedOutputEvidence::own_evidence_is_current(
                    &prefix, &[], candidate.native_output_witness().unwrap(), runtime, &live_handle, &mut expected,
                ).unwrap());
                // Full-prefix verification adds a visit and a native comparison
                // for every completed handler fact, after the own-evidence check.
                for fact in prefix.iter() {
                    expected.charge_external_work(1).unwrap();
                    assert_eq!(fact.source_currentness_in(runtime, &live_handle, &mut expected).unwrap().unwrap().movement(), Movement::Unmoved);
                }
                assert_eq!(available - currentness.remaining_work(), start - expected.remaining_work(),
                    "resolved pending marks must still verify the whole handler prefix");
            } else {
                assert!(matches!(result, Err(InputCutoffVerificationStop::PendingOutput(ref identity)) if identity == &a0),
                    "NoConsequence with a Dirty upstream must remain the exact typed deferral");
            }
            for handle in [before_handle, selected_handle, live_handle] {
                runtime.snapshots().release_snapshot(&handle).unwrap();
            }
        });
    });
}

fn register_dependent_alias(
    owner: &SourceInvalidationOwner,
    basis: &PositionedRelationalSnapshot,
    predecessor: &Arc<RecordedSettlementIdentity>,
    successor: &Arc<RecordedSettlementIdentity>,
    upstream: &Arc<RecordedSettlementIdentity>,
) {
    let relation = StableEqualityConsequence::native_actor_fixture(
        Arc::clone(predecessor),
        Arc::clone(successor),
        basis,
    );
    let prepared = owner
        .prepare_current_stable_settlement(
            SettlementRegistration {
                work_membership: None,
                identity: Arc::clone(successor),
                facts: RetainedSourceFacts::for_test(false, Arc::from([])),
                output_facts: None,
                read_basis: basis.clone(),
                stale_at_read_basis: OrdSet::new(),
                requirement: None,
                upstream: OrdSet::unit(Arc::clone(upstream)),
            },
            basis,
            relation,
            &mut owner.edit_admission(),
        )
        .expect("a clean dependent alias prepares at its actual native basis");
    drop(
        prepared
            .install()
            .unwrap_or_else(|_| panic!("the selected dependent alias installs")),
    );
}

fn context() -> PreparedDecisionReuseContext {
    PreparedDecisionReuseContext::new(
        WorthQueryProducerInputReuseContract::canonical_bitwise(
            WorthQueryDecisionContextDependencies::KEY,
        ),
        Some([11; 32]),
        None,
        None,
    )
    .unwrap()
}

/// Reuse metadata is descriptive fixture input; output truth and its sealed
/// witness come from the genuine generated publication. The actor's marks and
/// every cutoff comparison use real native snapshots, not injected answers.
fn candidate(
    performed: &RecordedOutput,
    identity: Arc<RecordedSettlementIdentity>,
    prefix: Arc<[WorthQueryApplicationObservedFact]>,
) -> RetainedInputCutoffCandidate {
    let boundary = CompletedHandlerFactBoundary::completed_for_test(prefix.len());
    RetainedInputCutoffCandidate::from_exact_cell(Arc::new(OnceLock::from(RecordedOutput {
        computation_source: ComputationSourceEvidence::for_test(false),
        native_prior_checkpoint: None,
        _retained_capacity: None,
        performed_origin: None,
        consumed_outputs: Arc::from([]),
        completed_decision_reuse: boundary
            .seal_decision_reuse(context(), DecisionContextUse::default().key_for_test()),
        completed_handler_facts: Some(boundary),
        prepared_input_reuse_key: Some(PreparedInputReuseKey::new(
            WorthQueryObservedSourceSelection::selecting_for_test(
                u64::try_from(prefix.len()).unwrap(),
            ),
            [11; 32],
            InstalledProducerEdition::for_test([12; 32]),
        )),
        native_output_witness: OnceLock::from(Arc::clone(
            performed.native_output_witness_cell().unwrap(),
        )),
        settlement_identity: identity,
        correspondence: Arc::clone(&performed.correspondence),
        source_identity: performed.source_identity,
        source_partition_identity: performed.source_partition_identity,
        producer_dependency_identity: performed.producer_dependency_identity,
        idempotency_key_identity: performed.idempotency_key_identity,
        mutable: Mutex::new(RecordedOutputMutable::new(
            None,
            Some(RetainedSourceFacts::for_test(false, prefix)),
            None,
            RecordedComputation::Absent(PriorAbsence::Restored),
        )),
    })))
}

#[test]
fn retained_registration_consumes_the_equal_successor_at_the_live_image() {
    with_generated_own_write(|world, _, performed, _| {
        let graph = world.application.runtime.primary_graph().unwrap();
        let handle = graph.integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        let record = performed.get().unwrap();
        let entity = world
            .selected_product()
            .resolve_entity(
                AccountStatus::reference(),
                "2".to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
            .entity_id();
        let status = AccountStatus::reference();
        let status = graph
            .layout()
            .field_locator(status.entity(), status.aspect(), status.field())
            .unwrap()
            .clone();
        let mut source = record.settlement_identity.source().clone();
        source.output_binding = OutputBindingIdentity::declared("Upstream");
        let a0 =
            RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 0);
        let a1 =
            RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 1);
        source.output_binding = OutputBindingIdentity::declared("Consumer");
        let consumer =
            RecordedSettlementIdentity::retain(&source, record.settlement_identity.coordinate(), 0);
        handle.with_runtime_mut(|runtime| {
            let (before_handle, before) = snapshot(runtime);
            register(
                owner,
                Arc::clone(&a0),
                Arc::from([field_fact(runtime, &before_handle, entity, status.clone())]),
                &before,
                OrdSet::new(),
            );
            write_field(runtime, entity, status.clone(), "retained-upstream-changed");
            let (live_handle, live) = snapshot(runtime);
            register_alias(
                owner,
                runtime,
                &live_handle,
                &live,
                entity,
                status,
                &a0,
                &a1,
            );
            assert_ne!(
                before, live,
                "registration must exercise Retained alignment"
            );
            assert!(matches!(
                currentness(owner, &live, &a0),
                SourceSettlementCurrentness::Dirty(_)
            ));
            // No registrant facts changed; only its historical upstream edge
            // could incorrectly reconstruct Pending from the old Dirty row.
            register(
                owner,
                Arc::clone(&consumer),
                Arc::from([]),
                &before,
                OrdSet::unit(Arc::clone(&a0)),
            );
            assert!(
                matches!(
                    currentness(owner, &live, &consumer),
                    SourceSettlementCurrentness::Clean
                ),
                "retained registration must consume the certified live consequence"
            );
            runtime
                .snapshots()
                .release_snapshot(&before_handle)
                .unwrap();
            runtime.snapshots().release_snapshot(&live_handle).unwrap();
        });
    });
}
