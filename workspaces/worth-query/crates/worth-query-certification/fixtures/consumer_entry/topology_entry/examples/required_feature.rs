//! Run with `cargo run -p worth-query-topology-entry --example required_feature`.
#![recursion_limit = "256"]
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation as installation,
    primary_graph,
};
use worth_query_topology_entry::*;

#[cfg(test)]
#[path = "required_feature/omissions.rs"]
mod omissions;
#[path = "required_feature/program.rs"]
mod program;
#[path = "required_feature/installation.rs"]
mod setup;
use program::{ExampleProgram, ExampleSchema, FinalConnection, Root};
type Application =
    installation::WorthQueryProgramApplicationRuntime<ExampleSchema, ExampleProgram<false>>;
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, ExampleSchema>;

fn main() {
    run();
}

fn run() {
    let _observation = primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let mut application = setup::install::<false>(setup::configuration(), || {}).unwrap();
    {
        let (scope, principal) = setup::authenticate(&application);
        let request = application.request(&principal, &scope);
        let first = settle(&request, &application);
        assert_eq!(first.producer_contacts_in_this_demand(), 1);
        let entity = first
            .outputs_of::<FinalPlanarOutputs>()
            .unwrap()
            .entity::<FinalAnchorOutput<ExampleSchema>>()
            .unwrap()
            .entity_id();
        assert_value(&request, 3); // authored y=1, final value=y+2
        drop(first);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let before = application.producer_contacts_on_this_thread_for_test();
        let (edited, writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: setup::length(9),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&91_u64)
            .execute_performed::<ExampleProgram<false>, Root>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            });
        let WorthQueryApplicationPerformedMutationOutcome::Performed(edited) = edited else {
            panic!("the synchronous source edit performs");
        };
        assert_eq!(
            application.producer_contacts_on_this_thread_for_test() - before,
            3,
            "the root and both declared dependents execute inside the source edit"
        );
        assert_eq!(
            writes.published_rows.len() + writes.omitted_publications,
            1,
            "source and required outputs share one actual Native publication"
        );
        assert_value(&request, 11); // already y+2 before any delivery demand
        let commit = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        assert_eq!(
            edited
                .receipt()
                .committed_product_publication()
                .composite_commit(),
            &commit
        );
        drop(source);
        let (preserved, delivery_writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                settle(&request, &application)
            });
        assert_eq!(preserved.producer_contacts_in_this_demand(), 0);
        assert_eq!(
            delivery_writes.published_rows.len() + delivery_writes.omitted_publications,
            0
        );
        assert_eq!(
            preserved
                .outputs_of::<FinalPlanarPreserveOutputs>()
                .unwrap()
                .entity::<FinalPreservedAnchorOutput<ExampleSchema>>()
                .unwrap()
                .entity_id(),
            entity
        );
        assert_value(&request, 11); // independently expected from the authored edit
        let (reused, reuse_writes) =
            worth_relational::facade::inspection::observe_maintenance_work(1, || {
                settle(&request, &application)
            });
        assert_eq!(reused.producer_contacts_in_this_demand(), 0);
        assert_eq!(
            reuse_writes.published_rows.len() + reuse_writes.omitted_publications,
            0
        );
        assert_eq!(
            reused
                .outputs_of::<FinalPlanarPreserveOutputs>()
                .unwrap()
                .entity::<FinalPreservedAnchorOutput<ExampleSchema>>()
                .unwrap()
                .entity_id(),
            entity
        );
        assert_eq!(
            application.producer_contacts_on_this_thread_for_test() - before,
            3
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            &commit
        );
        assert_value(&request, 11);
    }
    assert!(application.close_live_delivery().owner_terminal());
    application.close_conditional_runtime().unwrap();
}

fn settle(
    request: &Request<'_, '_, '_>,
    application: &Application,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<ExampleProgram<false>, FinalConnection>(application)
        .unwrap();
    for _ in 0..64 {
        if let WorthQueryApplicationOutputDemandProgress::Settled(output) =
            demand.advance(request).unwrap()
        {
            return output;
        }
    }
    panic!("the installed required pair must settle");
}

fn assert_value(request: &Request<'_, '_, '_>, expected: u64) {
    let output = request
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(output.rows().len(), 1);
    assert_eq!(output.rows()[0].value, setup::length(expected));
}

#[test]
fn initial_edit_preserve_uses_the_installed_pair() {
    main();
}
