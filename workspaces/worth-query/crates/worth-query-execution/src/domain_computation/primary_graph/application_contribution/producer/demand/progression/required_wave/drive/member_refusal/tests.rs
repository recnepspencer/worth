//! The advance's custody, not the row's origin tag, decides its refusal.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
#[test]
fn a_queue_refusal_uses_its_advance_custody_not_the_rows_caller_tag() {
    for caller_tag in [false, true] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let (_interest, ready) = registry.fixture_refusing_ready();
        let key = ready.key().clone();
        let mut frontier = RequiredFrontier::new();
        let mut admission = InvalidationEditAdmission::new(
            worth_relational::facade::mvcc::CompanionPreflightBudget {
                maximum_work_visits: 1_000_000,
                maximum_preparation_bytes: 8 * 1024 * 1024,
            },
        );
        frontier
            .insert(ready, caller_tag, None, &mut admission)
            .unwrap();
        let mut queue = QueuedRequiredRoots::new();
        queue.defer_caller();
        let mut custody = RequiredContinuations::<IdentityExecutionSchema>::default();
        let kind = WorthQueryOutputDemandDenialKind::ProducerUnavailable;
        let result = RefusingMember {
            registry: &registry,
            member: &mut frontier.members[0],
            row: RefusalRow::Member,
            custody: RefusalCustody::Queue(&mut custody),
        }
        .end(
            WorthQueryOutputDemandDenial::new(kind.clone(), "the queued member's own stop"),
            StopRecording::member(),
            &mut queue,
            &mut admission,
        );
        match result {
            Ok(RefusalEnd::Held) => {}
            Ok(RefusalEnd::QueueBudget) => panic!("the row's stop is not a request budget"),
            Err(stop) => panic!(
                "Queue custody owns this refusal regardless of the row tag: {:?}",
                stop.kind()
            ),
        }
        assert_eq!(registry.fixture_stop_kind(&key), Some(kind));
        assert!(matches!(frontier.members[0].state, MemberState::Pending));
        assert!(matches!(
            queue.take_caller(),
            Some(WorthQueryOutputDemandAdvance::Pending)
        ));
    }
}
#[test]
fn a_queue_budget_refusal_returns_its_typed_end() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (_interest, ready) = registry.fixture_refusing_ready();
    let mut frontier = RequiredFrontier::new();
    let mut admission =
        InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: 1_000_000,
            maximum_preparation_bytes: 8 * 1024 * 1024,
        });
    frontier.insert(ready, false, None, &mut admission).unwrap();
    let mut queue = QueuedRequiredRoots::new();
    queue.defer_caller();
    let mut custody = RequiredContinuations::<IdentityExecutionSchema>::default();
    let result = RefusingMember {
        registry: &registry,
        member: &mut frontier.members[0],
        row: RefusalRow::Member,
        custody: RefusalCustody::Queue(&mut custody),
    }
    .end(
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            "required custody refused",
        ),
        StopRecording::member(),
        &mut queue,
        &mut admission,
    );
    assert!(matches!(result, Ok(RefusalEnd::QueueBudget)));
    assert!(matches!(frontier.members[0].state, MemberState::Pending));
}

#[test]
fn a_caller_refusal_returns_its_stop_even_for_a_queue_origin_row() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (_interest, ready) = registry.fixture_refusing_ready();
    let key = ready.key().clone();
    let mut frontier = RequiredFrontier::new();
    let mut admission =
        InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: 1_000_000,
            maximum_preparation_bytes: 8 * 1024 * 1024,
        });
    frontier.insert(ready, false, None, &mut admission).unwrap();
    let mut queue = QueuedRequiredRoots::new();
    let kind = WorthQueryOutputDemandDenialKind::ProducerUnavailable;
    let result = RefusingMember::<IdentityExecutionSchema> {
        registry: &registry,
        member: &mut frontier.members[0],
        row: RefusalRow::Member,
        custody: RefusalCustody::Caller,
    }
    .end(
        WorthQueryOutputDemandDenial::new(kind.clone(), "the caller's own refusal"),
        StopRecording::member(),
        &mut queue,
        &mut admission,
    );
    assert!(matches!(result,Err(stop) if stop.kind()==kind));
    assert_eq!(registry.fixture_stop_kind(&key), Some(kind));
    assert!(!queue.draining());
}
