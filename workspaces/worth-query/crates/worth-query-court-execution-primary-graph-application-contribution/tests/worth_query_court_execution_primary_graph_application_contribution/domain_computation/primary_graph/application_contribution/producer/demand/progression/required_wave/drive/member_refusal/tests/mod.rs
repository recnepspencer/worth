//! The advance's custody, not the row's origin tag, decides its refusal.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::refusal_row::RefusalRowWorthQueryOutputDemandRegistryForTest;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::MemberState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::queued::QueuedRequiredRoots;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::drive::member_refusal::RefusalCustody;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::drive::member_refusal::RefusalEnd;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::drive::member_refusal::RefusalRow;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::drive::member_refusal::RefusingMember;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::required_continuations::RequiredContinuations;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::frontier::RequiredFrontier;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::drive::member_stop::StopRecording;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandAdvance;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
#[test]
fn a_queue_refusal_uses_its_advance_custody_not_the_rows_caller_tag() {
    for caller_tag in [false, true] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let (_interest, ready) = registry.fixture_refusing_ready();
        let key = ready.key_for_test().clone();
        let mut frontier = RequiredFrontier::new();

        frontier.insert(ready, caller_tag, None).unwrap();
        let mut queue = QueuedRequiredRoots::new_for_test();
        queue.defer_caller_for_test();
        let mut custody = RequiredContinuations::<IdentityExecutionSchema>::default();
        let kind = WorthQueryOutputDemandDenialKind::ProducerUnavailable;
        let result = RefusingMember {
            registry: &registry,
            member: &mut frontier.members[0],
            row: RefusalRow::Member,
            custody: RefusalCustody::Queue(&mut custody),
        }
        .end_for_test(
            WorthQueryOutputDemandDenial::new_for_test(
                kind.clone(),
                "the queued member's own stop",
            ),
            StopRecording::member_for_test(),
            &mut queue,
        );
        match result {
            Ok(end @ RefusalEnd::Held) => {
                assert!(end
                    .finish_for_test(|| panic!("a held queue row does not end its cursor"))
                    .is_none());
            }
            Ok(RefusalEnd::QueueBudget) => panic!("the row's stop is not a request budget"),
            Err(stop) => panic!(
                "Queue custody owns this refusal regardless of the row tag: {:?}",
                stop.kind()
            ),
        }
        assert_eq!(registry.fixture_stop_kind(&key), Some(kind));
        assert!(matches!(frontier.members[0].state, MemberState::Pending));
        assert!(matches!(
            queue.take_caller_for_test(),
            Some(WorthQueryOutputDemandAdvance::Pending)
        ));
    }
}
#[test]
fn a_queue_budget_refusal_returns_its_typed_end() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (_interest, ready) = registry.fixture_refusing_ready();
    let mut frontier = RequiredFrontier::new();

    frontier.insert(ready, false, None).unwrap();
    let mut queue = QueuedRequiredRoots::new_for_test();
    queue.defer_caller_for_test();
    let mut custody = RequiredContinuations::<IdentityExecutionSchema>::default();
    let result = RefusingMember {
        registry: &registry,
        member: &mut frontier.members[0],
        row: RefusalRow::Member,
        custody: RefusalCustody::Queue(&mut custody),
    }
    .end_for_test(
        WorthQueryOutputDemandDenial::new_for_test(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            "required custody refused",
        ),
        StopRecording::member_for_test(),
        &mut queue,
    );
    let end = result.unwrap_or_else(|_| panic!("queue custody returns its budget end"));
    let mut finished = false;
    let returned = end.finish_for_test(|| {
        finished = true;
        WorthQueryOutputDemandAdvance::Pending
    });
    assert!(
        finished,
        "the budget door invokes the owner's canonical finish once"
    );
    assert!(matches!(
        returned,
        Some(WorthQueryOutputDemandAdvance::Pending)
    ));
    assert!(matches!(frontier.members[0].state, MemberState::Pending));
}

#[test]
fn a_caller_refusal_returns_its_stop_even_for_a_queue_origin_row() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (_interest, ready) = registry.fixture_refusing_ready();
    let key = ready.key_for_test().clone();
    let mut frontier = RequiredFrontier::new();

    frontier.insert(ready, false, None).unwrap();
    let mut queue = QueuedRequiredRoots::new_for_test();
    let kind = WorthQueryOutputDemandDenialKind::ProducerUnavailable;
    let result = RefusingMember::<IdentityExecutionSchema> {
        registry: &registry,
        member: &mut frontier.members[0],
        row: RefusalRow::Member,
        custody: RefusalCustody::Caller,
    }
    .end_for_test(
        WorthQueryOutputDemandDenial::new_for_test(kind.clone(), "the caller's own refusal"),
        StopRecording::member_for_test(),
        &mut queue,
    );
    let mut finished = false;
    let result = result.map(|end| {
        end.finish_for_test(|| {
            finished = true;
            WorthQueryOutputDemandAdvance::Pending
        })
    });
    assert!(matches!(result,Err(stop) if stop.kind()==kind));
    assert!(
        !finished,
        "caller custody returns its own stop before the cursor finish"
    );
    assert_eq!(registry.fixture_stop_kind(&key), Some(kind));
    assert!(!queue.draining_for_test());
}
