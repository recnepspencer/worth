//! Actual source custody can be lost before and after its one real successor.
use super::super::{authentication::*, host, output_program::OutputProgram};
use super::{authoring::Edit, calculation, program::Root};
use worth_query_execution::facade::primary_graph::WorthQueryExecutionObservation;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

pub fn retry_before_and_after_successor(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let application = host::install_with_output_limits(if worker {
        host::worker_limits()
    } else {
        host::installation_limits()
    });
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let contacts_before = application
        .runtime()
        .producer_contacts_on_this_thread_for_test();
    let (performed, writes) = observe_maintenance_work(8, || {
        request
            .advanced()
            .mutate(Edit {
                key: "numeric".into(),
                value: 3,
            })
            .without_source()
            .idempotency(&"born-stale-retry".to_owned())
            .execute_performed::<OutputProgram, Root>(
                &application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("real synchronous scalar source")
    };
    let source = performed.receipt().clone();
    let own_effect = super::courts::commit_evidence::paired_edit(
        &application,
        &source,
        3,
        writes.published_rows.len() + writes.omitted_publications,
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test()
            - contacts_before,
    );
    assert!(
        own_effect.is_some_and(|facts| facts > 0),
        "Native marks the actual in-edit run born stale"
    );
    // Lose the real pending preparation before any delivery execution. A new
    // public demand must resume the actual committed producer, not recreate it.
    drop(performed);
    let (successor, contacts, commits) = settle(&application);
    assert_eq!((contacts, commits), (1, 1));
    let successor = successor.expect("the real successor publication retains its receipt");
    assert_eq!(
        successor.commit_reference().parents.as_slice(),
        &[source.commit_reference().commit_id]
    );
    assert_ne!(
        successor.idempotency_binding().key_identity(),
        source.idempotency_binding().key_identity()
    );
    equivalent_at_current(&application, &successor);
    // The settled handle is already dropped. Another delivery after publication
    // must neither execute a producer nor create another Native/World commit.
    let (rejoined, contacts, commits) = settle(&application);
    assert_eq!((contacts, commits), (0, 0));
    if let Some(rejoined) = rejoined {
        assert_eq!(rejoined.commit_reference(), successor.commit_reference());
    }
    equivalent_at_current(&application, &successor);
    assert_values(&application);
}

fn settle(
    application: &host::OutputHost,
) -> (Option<WorthQueryApplicationCommitReceipt>, u64, usize) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let before = application
        .runtime()
        .producer_contacts_on_this_thread_for_test();
    let (receipt, writes) = observe_maintenance_work(8, || {
        let mut demand = request
            .demand(calculation::Demand {
                key: "numeric".into(),
            })
            .start_in_program::<OutputProgram, Root>(application)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&request).unwrap()
        else {
            panic!("actual scalar demand settles")
        };
        settled.application_commit_receipt().cloned()
    });
    assert_values(application);
    (
        receipt,
        application
            .runtime()
            .producer_contacts_on_this_thread_for_test()
            - before,
        writes.published_rows.len() + writes.omitted_publications,
    )
}

fn equivalent_at_current(
    application: &host::OutputHost,
    successor: &WorthQueryApplicationCommitReceipt,
) {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolution;
    let ((), writes) = observe_maintenance_work(8, || {
        application
            .with_host_advancement(|phase| {
                let selected = application
                    .runtime()
                    .on_branch(application.current_world())
                    .select(&phase)
                    .unwrap();
                let execution = phase
                    .checked_execution_request_for(application.runtime().product_runtime())
                    .unwrap();
                let provider = application
                    .runtime()
                    .runtime_for_test()
                    .primary_provider_for_test();
                let result = provider
                    .resolve_idempotency_binding_at_product_for_test(
                        &execution,
                        successor.idempotency_binding(),
                        &selected.product().publication_binding_for_test(),
                    )
                    .unwrap();
                let WorthQueryProviderIdempotencyResolution::Equivalent(committed) = result else {
                    panic!("retry resolves the real published successor Equivalent")
                };
                assert_eq!(
                    committed.commit_reference_for_test(),
                    successor.commit_reference()
                );
            })
            .unwrap()
    });
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
}

fn assert_values(application: &host::OutputHost) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let data = request
        .query(super::queries::data::Read {
            key: "numeric".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(data.rows().len(), 1);
    assert_eq!((data.rows()[0].input, data.rows()[0].scalar), (5, Some(3)));
    let output = request
        .query(super::output::query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            super::super::causal_inspection::capability::Inspect::reference(),
            super::super::causal_inspection::capability::InspectCausalEvidence::reference(),
            super::super::causal_inspection::capability::Input::at("numeric"),
        )
        .unwrap();
    assert_eq!(output.rows().len(), 1);
    assert_eq!(output.rows()[0].total, Some(38));
}
