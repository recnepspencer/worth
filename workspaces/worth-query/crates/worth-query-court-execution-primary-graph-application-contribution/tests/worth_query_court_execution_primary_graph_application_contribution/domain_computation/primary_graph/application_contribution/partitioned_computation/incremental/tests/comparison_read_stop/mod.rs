//! A retained prior reaches the real incremental comparator in an admitted projection.
use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::*;
use worth_query_execution::facade::application_contribution::WorthQueryComputationReadDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[test]
fn incremental_comparison_keeps_the_retained_fact_read_stop() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let installed = installed(StatusRead::Gather(1), sum);
    let prior = prior_of(first_run(&world, &installed), false);
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        None,
        source.token(),
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let policy = worth_query_execution::court_doors_for_test::domain_computation::request_execution::test_placement::test_policy(
        std::num::NonZeroUsize::MIN,
        None,
    );
    let at_read = std::rc::Rc::new(std::cell::Cell::new(None));
    let fired = std::rc::Rc::clone(&at_read);
    let mut observer = None;
    let mut source = Some(source);
    let next = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::tests::projected_attempt::attempt_admitted_placed_observing(
        &world,
        &installed,
        Some(prior),
        &request,
        RuntimeWorldExecutionPlacement::Serial(policy),
        |_| {
            let source = source.take().unwrap();
            let fired = std::rc::Rc::clone(&fired);
            observer = Some(worth_query_execution::test_read_stops::observe_consults_for_test(
                move |at| {
                    if worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::observed::retained_fact_comparison_is_active_for_test()
                        && fired.get().is_none()
                    {
                        fired.set(Some(at));
                        source.cancel();
                    }
                },
            ));
        },
    );
    assert!(
        worth_query_execution::test_read_stops::consult_count_for_test(observer.as_ref().unwrap())
            >= at_read
                .get()
                .expect("the retained fact comparison consult was reached")
    );
    drop(observer);
    assert_eq!(next.outcome, Err(WorthQueryPartitionedComputationDenial::Read(WorthQueryComputationReadDenial::RetainedFact(worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind::Cancelled))));
    assert!(
        next.gathered.is_empty(),
        "a stopped comparison must not rebuild"
    );
    assert!(next.runs.is_empty());
}
