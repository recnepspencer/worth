use super::*;
use crate::domain_computation::{
    execution_observation::WorthQueryExecutionObservation,
    primary_graph::fact_verification_on_this_thread_for_test as take,
};

#[test]
fn nested_cutoffs_and_unwinding_keep_their_own_decisive_gate() {
    let _listener = WorthQueryExecutionObservation::install_on_this_thread();
    take();
    let outer = Scope::begin("outer", Gate::Candidate);
    reach(Gate::RegistryCustody);
    let unwind = std::panic::catch_unwind(|| {
        let _inner = Scope::begin("inner", Gate::PreparedKey);
        reach(Gate::SelectedMembership);
        panic!("stop the inner invocation");
    });
    assert!(unwind.is_err());
    finish(Outcome::Fresh);
    drop(outer);
    let receipts = take();
    let [Some(inner), Some(outer), None, ..] = receipts.input_cutoffs else {
        panic!("two complete receipts expected: {receipts:?}");
    };
    assert_eq!(
        (inner.producer, inner.gate, inner.outcome),
        ("inner", Gate::SelectedMembership, Outcome::Stopped)
    );
    assert_eq!(
        (outer.producer, outer.gate, outer.outcome),
        ("outer", Gate::RegistryCustody, Outcome::Fresh)
    );
    assert_eq!(receipts.input_cutoffs_omitted, 0);
}

#[test]
fn receipts_are_bounded_and_end_with_the_existing_listener() {
    {
        let _listener = WorthQueryExecutionObservation::install_on_this_thread();
        take();
        for _ in 0..35 {
            let _scope = Scope::begin("bounded", Gate::ComparableFacts);
            super::super::registry_custody(
                super::super::WorthQueryRegistryCustodyTransitionForTest::PostingRelease,
                1,
                2,
            );
        }
        let receipts = take();
        assert!(receipts.input_cutoffs.iter().all(Option::is_some));
        assert_eq!(receipts.input_cutoffs_omitted, 3);
        assert!(receipts.registry_custody.iter().all(Option::is_some));
        assert_eq!(receipts.registry_custody_omitted, 3);
        let _scope = Scope::begin("cleared", Gate::Candidate);
    }
    let _unobserved = Scope::begin("unobserved", Gate::ExecutionRequest);
    reach(Gate::PreparedKey);
    drop(_unobserved);
    let receipts = take();
    assert!(receipts.input_cutoffs.iter().all(Option::is_none));
    assert_eq!(receipts.input_cutoffs_omitted, 0);
    assert!(receipts.registry_custody.iter().all(Option::is_none));
    assert_eq!(receipts.registry_custody_omitted, 0);
}
