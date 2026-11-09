//! Paired source execution, real delivery and cold reuse for co-committed edits.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation;
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_relational::facade::inspection::observe_maintenance_work;
mod cold;

pub(in super::super) fn publish_and_deliver<B>(
    application: &host::OutputHost,
    expected: u64,
    execute: impl FnOnce() -> WorthQueryApplicationPerformedMutationOutcome<
        NumericSchema,
        B,
        OutputProgram,
        Root,
    >,
) -> WorthQueryApplicationCommitReceipt
where
    B: ApplicationMutationBinding<NumericSchema>
        + WorthQueryApplicationRequiredOutputSource<
            NumericSchema,
            super::super::output_program::Connection,
        >,
{
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let before = application
        .runtime()
        .producer_contacts_on_this_thread_for_test();
    let (source, writes) = observe_maintenance_work(8, execute);
    let WorthQueryApplicationPerformedMutationOutcome::Performed(source) = source else {
        panic!("the actual co-committed source must perform")
    };
    assert!(
        source.receipt().aftermath_causality().is_none(),
        "a co-committed source is an authored edit, not a recorded correction"
    );
    assert_eq!(
        read_calculation(application),
        expected,
        "derived value is updated before start_required_outputs"
    );
    assert_eq!(
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test()
            - before,
        1,
        "the producer executes exactly once inside the source edit"
    );
    assert_eq!(
        writes.published_rows.len() + writes.omitted_publications,
        1,
        "one real Native commit across all branches"
    );
    let original = source.receipt().clone();
    settle(application, source);
    assert_eq!(read_calculation(application), expected);
    cold::assert_reopened_ready(application, expected);
    original
}
