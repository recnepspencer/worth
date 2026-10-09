//! Sorted-run lookup retains full predicate identity at its range boundaries.
use super::super::{run::Run, RetainedFact};
use super::{control, key, Body};
use worth_execution::ExecutionAllocationPolicy as Policy;

#[test]
fn full_key_run_lookup_preserves_endpoints_and_interior_matches() {
    let policy = control(Policy::SystemAllocation);
    let empty = Run::<Body>::empty(0, policy).unwrap();
    assert!(empty
        .locate(&key(b"same", None, policy), policy)
        .unwrap()
        .is_none());

    let mut run = Run::empty(3, policy).unwrap();
    for (ordinal, limit) in [2, 4, 6].into_iter().enumerate() {
        *run.slots[ordinal].borrow_mut() = Some(RetainedFact {
            key: key(b"same", Some((b"predicate", limit)), policy),
            value: Body {
                scalar: limit as u64,
                roles: 1,
            },
            ordinal,
        });
        run.used += 1;
    }
    for limit in [2, 4, 6] {
        let probe = key(b"same", Some((b"predicate", limit)), policy);
        let found = run.locate(&probe, policy).unwrap().unwrap().borrow();
        assert_eq!(found.as_ref().unwrap().value.scalar, limit as u64);
    }
    for (locator, predicate) in [
        (b"sama".as_slice(), Some((b"predicate".as_slice(), 4))),
        (b"samf".as_slice(), Some((b"predicate".as_slice(), 4))),
        (b"same".as_slice(), None),
        (b"same".as_slice(), Some((b"predicatd".as_slice(), 4))),
        (b"same".as_slice(), Some((b"predicatf".as_slice(), 4))),
        (b"same".as_slice(), Some((b"predicate".as_slice(), 1))),
        (b"same".as_slice(), Some((b"predicate".as_slice(), 3))),
        (b"same".as_slice(), Some((b"predicate".as_slice(), 5))),
        (b"same".as_slice(), Some((b"predicate".as_slice(), 7))),
    ] {
        assert!(run
            .locate(&key(locator, predicate, policy), policy)
            .unwrap()
            .is_none());
    }
}
