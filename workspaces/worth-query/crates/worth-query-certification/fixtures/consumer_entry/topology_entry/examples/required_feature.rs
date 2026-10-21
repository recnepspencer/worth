//! Run with `cargo run -p worth-query-topology-entry --example required_feature`.
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
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn run() {
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
            .unwrap();
        drop(source);
        let preserved = settle(&request, &application);
        assert_eq!(preserved.producer_contacts_in_this_demand(), 1);
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
