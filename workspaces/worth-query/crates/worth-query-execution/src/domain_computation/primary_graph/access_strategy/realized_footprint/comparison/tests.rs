//! Root-free adversarial evidence for the erased fallible comparison boundary.
use super::with_checkpoint;
#[test]
fn swallowed_checkpoint_preserves_first_owned_cause_and_stops_admission() {
    #[derive(Debug, Eq, PartialEq)]
    struct Denial(String);
    let mut contacts = 0;
    let result = with_checkpoint(
        &mut |_| {
            contacts += 1;
            Err(Denial("original-domain-stop".into()))
        },
        &mut |checkpoint| {
            assert!(!checkpoint(32));
            assert!(!checkpoint(64));
            true // hostile erased callee swallows both refusals
        },
    );
    assert_eq!(result, Err(Denial("original-domain-stop".into())));
    assert_eq!(contacts, 1);
}
#[test]
fn final_poll_can_refuse_an_otherwise_equal_or_different_comparison() {
    for equal in [false, true] {
        assert_eq!(
            with_checkpoint(&mut |_| Err("final-stop"), &mut |_| equal),
            Err("final-stop")
        );
    }
}

#[test]
fn swallowed_resource_refusals_preserve_first_cause_for_both_boolean_results() {
    use super::with_resources;
    use crate::domain_computation::primary_graph::application_query::ProjectionContentStorageFault as Fault;
    use std::cell::Cell;
    for first in 0..3 {
        for equal in [false, true] {
            let work = Cell::new(0);
            let memory = Cell::new(0);
            let physical = Cell::new(0);
            let result = with_resources(
                &mut |_| {
                    work.set(work.get() + 1);
                    Err("work".to_owned())
                },
                &mut |_| {
                    memory.set(memory.get() + 1);
                    Err("memory".to_owned())
                },
                &mut |_| {
                    physical.set(physical.get() + 1);
                    "physical".to_owned()
                },
                &mut |checkpoint, scratch| {
                    match first {
                        0 => assert!(!checkpoint(1)),
                        1 => assert!((scratch.reserve)(32).is_err()),
                        2 => (scratch.fault)(Fault::AllocationFailed),
                        _ => unreachable!(),
                    }
                    assert!(!checkpoint(99));
                    assert!((scratch.reserve)(64).is_err());
                    (scratch.fault)(Fault::CapacityOverflow);
                    equal // all original callbacks are now fenced
                },
            );
            assert_eq!(
                result,
                Err(["work", "memory", "physical"][first].to_owned())
            );
            assert_eq!(
                [work.get(), memory.get(), physical.get()],
                std::array::from_fn(|i| usize::from(i == first))
            );
        }
    }
}
#[test]
fn successful_resource_bridge_requires_original_final_poll_for_true_and_false() {
    for equal in [false, true] {
        assert_eq!(
            super::with_resources(
                &mut |work| if work == 0 { Err("final") } else { Ok(()) },
                &mut |_| Ok(None),
                &mut |_| "physical",
                &mut |checkpoint, scratch| {
                    assert!(checkpoint(1));
                    assert!((scratch.reserve)(32).is_ok());
                    equal
                },
            ),
            Err("final")
        );
    }
}
