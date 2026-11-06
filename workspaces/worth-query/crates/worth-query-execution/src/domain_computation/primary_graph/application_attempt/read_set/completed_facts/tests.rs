//! Descriptive builder ordering, not installed Query admission or graph authority.
use super::{admit_completed_facts, WorthQueryApplicationObservedFact as Fact};
use std::{cell::RefCell, rc::Rc};
use worth_execution::ExecutionAllocationPolicy;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_relational::facade::identity::{EntityId, PartitionId};
use {
    crate::domain_computation::primary_graph::vocabulary::application_attempt::attempt_denial::WorthQueryApplicationAttemptDenial,
    crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind as Kind,
};

type Events = Rc<RefCell<Vec<&'static str>>>;

struct Values {
    events: Events,
}

struct TrackedIterator {
    events: Events,
    values: std::vec::IntoIter<Fact>,
}

impl IntoIterator for Values {
    type Item = Fact;
    type IntoIter = TrackedIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.events.borrow_mut().push("into");
        TrackedIterator {
            events: self.events,
            values: vec![fact(17), fact(31)].into_iter(),
        }
    }
}

impl Iterator for TrackedIterator {
    type Item = Fact;

    fn next(&mut self) -> Option<Self::Item> {
        self.events.borrow_mut().push("next");
        self.values.next()
    }
}

impl Drop for TrackedIterator {
    fn drop(&mut self) {
        self.events.borrow_mut().push("drop");
    }
}

fn fact(id: u64) -> Fact {
    Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), id, 1),
    }
}

#[test]
fn literal_order_keeps_conversion_after_admission_and_drop_before_final_check() {
    let events = Events::default();
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let facts = admit_completed_facts(
        2,
        Values {
            events: Rc::clone(&events),
        },
        ExecutionAllocationPolicy::SystemAllocation,
        &request,
        "literal completion",
        || {
            events.borrow_mut().push("authority");
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        *events.borrow(),
        [
            "authority",
            "into",
            "next",
            "authority",
            "next",
            "authority",
            "next",
            "drop",
            "authority",
        ]
    );
    let ids: Vec<_> = facts
        .iter()
        .map(|fact| match fact {
            Fact::SourceEntity { entity_id } => *entity_id,
            other => panic!("unexpected completed fact: {other:?}"),
        })
        .collect();
    assert_eq!(
        ids,
        [
            EntityId::new(PartitionId::main(), 17, 1),
            EntityId::new(PartitionId::main(), 31, 1)
        ]
    );
}

#[test]
fn every_authority_refusal_keeps_its_exact_subject_and_iterator_drop_position() {
    let expected: [&[&str]; 4] = [
        &["authority"],
        &["authority", "into", "next", "authority", "drop"],
        &[
            "authority",
            "into",
            "next",
            "authority",
            "next",
            "authority",
            "drop",
        ],
        &[
            "authority",
            "into",
            "next",
            "authority",
            "next",
            "authority",
            "next",
            "drop",
            "authority",
        ],
    ];
    for (refuse_at, expected) in (1..=4).zip(expected) {
        let events = Events::default();
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let mut checks = 0;
        let refused = admit_completed_facts(
            2,
            Values {
                events: Rc::clone(&events),
            },
            ExecutionAllocationPolicy::SystemAllocation,
            &request,
            "completion subject",
            || {
                events.borrow_mut().push("authority");
                checks += 1;
                if checks == refuse_at {
                    Err(WorthQueryApplicationAttemptDenial::new(
                        Kind::DecisionDependencyMismatch,
                        "literal injected refusal",
                    ))
                } else {
                    Ok(())
                }
            },
        )
        .expect_err("initial, per-fact and final refusals cannot complete");
        assert_eq!(refused.kind(), Kind::DecisionDependencyMismatch);
        assert_eq!(refused.subject(), "literal injected refusal");
        assert_eq!(*events.borrow(), expected, "check {refuse_at}");
    }
}

#[test]
fn stopped_allocation_does_not_convert_or_advance_the_iterator() {
    let events = Events::default();
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    cancellation.cancel();
    let refused = admit_completed_facts(
        2,
        Values {
            events: Rc::clone(&events),
        },
        ExecutionAllocationPolicy::SystemAllocation,
        &request,
        "cancelled completion",
        || {
            events.borrow_mut().push("authority");
            Ok(())
        },
    )
    .expect_err("the original stopped request cannot admit backing");
    assert_eq!(refused.kind(), Kind::CurrentAuthorityDenied);
    assert_eq!(refused.subject(), "cancelled completion");
    assert_eq!(*events.borrow(), ["authority"]);
}
