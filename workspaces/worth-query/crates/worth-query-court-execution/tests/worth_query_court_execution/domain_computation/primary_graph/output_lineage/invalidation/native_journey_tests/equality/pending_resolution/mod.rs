//! Pending-edge resolution does not replace the cutoff's native verification.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::tests::TestsWorthQueryObservedSourceSelectionForTest;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::equality::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use std::sync::Mutex;
use std::sync::OnceLock;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::handler_fact_boundary::CompletedHandlerFactBoundary;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::decision_reuse::PreparedDecisionReuseContext;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::registry::semantic_edition::InstalledProducerEdition;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
use worth_query_execution::facade::application_contribution::WorthQueryDecisionContextDependencies;
use worth_query_execution::facade::application_contribution::WorthQueryProducerInputReuseContract;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::membership::WorthQueryObservedSourceSelection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::verification::InputCutoffDecision;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::verification::refusal::InputCutoffVerificationStop;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::selected_basis::PreparedInputCutoffBasis;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::RetainedInputCutoffCandidate;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_generated_own_write;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_reuse_key::PreparedInputReuseKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutputMutable;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::handler::decision_context_use::DecisionContextUse;

pub enum Successor {
    Equal,
    Unequal,
    Dirty,
}

#[test]
fn equal_clean_successor_passes_the_cutoff_and_verifies_the_full_handler_prefix() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    cutoff_journey(request_execution, Successor::Equal);
}

#[test]
fn clean_unequal_or_dirty_successor_still_defers_at_the_cutoff() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for successor in [Successor::Unequal, Successor::Dirty] {
        cutoff_journey(request_execution, successor);
    }
}

pub fn cutoff_journey(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    successor: Successor,
) {
    with_generated_own_write(|world, _, performed, _| {
        let graph = (*world.application.runtime_for_test())
            .primary_graph_for_test()
            .unwrap();
        let handle = graph.integration_handle_for_test();
        let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
        let record = performed.get().unwrap();
        let entity = world
            .application
            .with_host_advancement(|host_phase| {
                world
                    .selected_product(&host_phase.test_host_request_for_test())
                    .resolve_entity(
                        &host_phase,
                        AccountStatus::at("AccountStatus"),
                        "2".to_owned(),
                        &live_scope(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
                    .entity_id_for_test()
            })
            .expect("installed host call opens");
        let status = AccountStatus::at("AccountStatus");
        let status = graph
            .layout_for_test()
            .field_locator_for_test(status.entity(), status.aspect(), status.field())
            .unwrap()
            .clone();
        let label = AccountLabel::at("AccountLabel");
        let label = graph
            .layout_for_test()
            .field_locator_for_test(label.entity(), label.aspect(), label.field())
            .unwrap()
            .clone();
        let mut source = (*record.settlement_identity_for_test())
            .source_for_test()
            .clone();
        (*source.output_binding_mut_for_test()) =
            OutputBindingIdentity::declared_for_test("Upstream");
        let a0 = RecordedSettlementIdentity::retain_for_test(
            &source,
            (*record.settlement_identity_for_test()).coordinate_for_test(),
            0,
        );
        let a1 = RecordedSettlementIdentity::retain_for_test(
            &source,
            (*record.settlement_identity_for_test()).coordinate_for_test(),
            1,
        );
        (*source.output_binding_mut_for_test()) =
            OutputBindingIdentity::declared_for_test("Consumer");
        let consumer = RecordedSettlementIdentity::retain_for_test(
            &source,
            (*record.settlement_identity_for_test()).coordinate_for_test(),
            0,
        );
        handle.with_runtime_for_test(request_execution, |runtime| {
            let (before_handle, before) = snapshot(runtime);
            let prefix: Arc<[_]> = Arc::from([field_fact(request_execution, runtime, &before_handle, entity, label)]);
            register(owner, Arc::clone(&a0), Arc::from([field_fact(request_execution, runtime, &before_handle, entity, status.clone())]), &before, OrdSet::new());
            // b has an authentic equality chain. Both b rows become pending
            // when a changes; a's later equality clears them one level only,
            // leaving c's pending edge to b's predecessor in place.
            (*source.output_binding_mut_for_test()) = OutputBindingIdentity::declared_for_test("DownstreamOutput");
            let b = RecordedSettlementIdentity::retain_for_test(&source, (*record.settlement_identity_for_test()).coordinate_for_test(), 0);
            let b_successor = RecordedSettlementIdentity::retain_for_test(&source, (*record.settlement_identity_for_test()).coordinate_for_test(), 1);
            register(owner, Arc::clone(&b), Arc::from([]), &before, OrdSet::unit(Arc::clone(&a0)));
            register_dependent_alias(owner, &before, &b, &b_successor, &a0);
            register(owner, Arc::clone(&consumer), Arc::clone(&prefix), &before, OrdSet::unit(Arc::clone(&b)));
            write_field(request_execution, runtime, entity, status.clone(), "cutoff-upstream-changed");
            let (selected_handle, selected) = snapshot(runtime);
            match successor {
                Successor::Equal | Successor::Dirty => register_alias(request_execution, owner, runtime, &selected_handle, &selected, entity, status.clone(), &a0, &a1),
                Successor::Unequal => register(owner, Arc::clone(&a1), Arc::from([field_fact(request_execution, runtime, &selected_handle, entity, status.clone())]), &selected, OrdSet::new()),
            }
            if matches!(successor, Successor::Dirty) {
                write_field(request_execution, runtime, entity, status, "cutoff-successor-dirty");
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
                assert!(matches!(owner.consumed_output_currentness_for_test(
                    &live, &b,
                ).unwrap(),
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::ConsumedOutputCurrentness::CanonicallyEqualClean(identity)
                        if identity == b_successor),
                    "the cutoff's exact pending edge must have a certified equal clean successor");
            }
            if !matches!(successor, Successor::Equal) {
                // The cutoff must ask the Dirty predecessor directly: a clean
                // unrelated successor is not a consequence for this identity.
                register(owner, Arc::clone(&consumer), Arc::clone(&prefix), &live, OrdSet::unit(Arc::clone(&a0)));
                assert_pending(owner, &live, &consumer, &a0);
                let upstream = owner.consumed_output_currentness_for_test(
                    &live, &a0,
                ).unwrap();
                use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::derived::ConsumedOutputCurrentness;
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

let basis = PreparedInputCutoffBasis::prepare_for_test(runtime, &live_handle).unwrap();

let result = candidate.clone().verify_for_reuse_for_test(
                request_execution, candidate.prepared_input_key_for_test().unwrap().continued_by_republication_for_test(), context, None,
                runtime, &basis, owner,
            );
            if matches!(successor, Successor::Equal) {
                assert!(matches!(result, Ok(InputCutoffDecision::Reuse(_))),
                    "the resolved pending edge must reach full verification");

assert!(ConsumedOutputEvidence::own_evidence_is_current_for_test(
request_execution,                    &prefix, &[], candidate.native_output_witness_for_test().unwrap(), runtime, &live_handle,
                ).unwrap());
                // Full-prefix verification adds a visit and a native comparison
                // for every completed handler fact, after the own-evidence check.
                for fact in prefix.iter() {

assert_eq!(fact.source_currentness_in_for_test(runtime, request_execution, &live_handle).unwrap().unwrap().movement_for_test(), Movement::Unmoved);
                }
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

pub fn register_dependent_alias(
    owner: &SourceInvalidationOwner,
    basis: &PositionedRelationalSnapshot,
    predecessor: &Arc<RecordedSettlementIdentity>,
    successor: &Arc<RecordedSettlementIdentity>,
    upstream: &Arc<RecordedSettlementIdentity>,
) {
    let relation = StableEqualityConsequence::native_actor_fixture_for_test(
        Arc::clone(predecessor),
        Arc::clone(successor),
        basis,
    );
    let prepared = owner
        .prepare_current_stable_settlement_for_test(
            SettlementRegistration::new_for_test(
                None,
                None,
                Arc::clone(successor),
                RetainedSourceFacts::for_test_for_test(false, Arc::from([])),
                None,
                basis.clone(),
                OrdSet::new(),
                None,
                OrdSet::unit(Arc::clone(upstream)),
            ),
            basis,
            relation,
        )
        .expect("a clean dependent alias prepares at its actual native basis");
    drop(
        prepared
            .install_for_test()
            .unwrap_or_else(|_| panic!("the selected dependent alias installs")),
    );
}

pub fn context() -> PreparedDecisionReuseContext {
    PreparedDecisionReuseContext::new_for_test(
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
pub fn candidate(
    performed: &RecordedOutput,
    identity: Arc<RecordedSettlementIdentity>,
    prefix: Arc<[WorthQueryApplicationObservedFact]>,
) -> RetainedInputCutoffCandidate {
    let boundary = CompletedHandlerFactBoundary::completed_for_test(prefix.len());
    RetainedInputCutoffCandidate::from_exact_cell_for_test(Arc::new(OnceLock::from({
        // Bound in the original literal order, passed in the door's.
        let (
            producing_implementation,
            required_publication,
            computation_source,
            native_prior_checkpoint,
            performed_origin,
            consumed_outputs,
            completed_decision_reuse,
            completed_handler_facts,
            prepared_input_reuse_key,
            native_output_witness,
            settlement_identity,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            mutable,
        ) = (
            None,
            None,
            ComputationSourceEvidence::for_test(false),
            None,
            None,
            Arc::from([]),
            boundary.seal_decision_reuse_for_test(
                context(),
                DecisionContextUse::default().key_for_test(),
            ),
            Some(boundary),
            Some(PreparedInputReuseKey::new_for_test(
                WorthQueryObservedSourceSelection::selecting_for_test(
                    u64::try_from(prefix.len()).unwrap(),
                ),
                [11; 32],
                InstalledProducerEdition::for_test_for_test([12; 32]),
            )),
            OnceLock::from(Arc::clone(
                performed.native_output_witness_cell_for_test().unwrap(),
            )),
            identity,
            Arc::clone(performed.correspondence_for_test()),
            (*performed.source_identity_for_test()),
            (*performed.source_partition_identity_for_test()),
            (*performed.producer_dependency_identity_for_test()),
            (*performed.idempotency_key_identity_for_test()),
            Mutex::new(RecordedOutputMutable::new_for_test(
                None,
                Some(RetainedSourceFacts::for_test_for_test(false, prefix)),
                RecordedComputation::Absent(PriorAbsence::Restored),
            )),
        );
        RecordedOutput::new_for_test(
            producing_implementation,
            required_publication,
            computation_source,
            native_prior_checkpoint,
            performed_origin,
            consumed_outputs,
            completed_handler_facts,
            completed_decision_reuse,
            prepared_input_reuse_key,
            native_output_witness,
            settlement_identity,
            correspondence,
            source_identity,
            source_partition_identity,
            producer_dependency_identity,
            idempotency_key_identity,
            mutable,
            None,
        )
    })))
}
#[path = "live_image_registration/mod.rs"]
pub mod live_image_registration;
