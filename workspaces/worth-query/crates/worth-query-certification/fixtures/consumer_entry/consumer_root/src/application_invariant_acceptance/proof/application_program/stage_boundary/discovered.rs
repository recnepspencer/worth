//! Each original discovered root yields before its dependent discovery starts.
use super::*;
use crate::ConsumerDiscoveredProgramRoot;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationDiscoveredMutationOutcome,
    WorthQueryDiscoveredProgramOutputProgress as DiscoveredProgress,
};

#[test]
fn discovered_roots_yield_before_children_and_finish_once_with_fresh_requests() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let mut outputs = {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let input = PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(2),
        };
        let performed = request
            .advanced()
            .mutate(input)
            .expect_source(source)
            .idempotency(&10_723)
            .execute_performed_discovered::<crate::ConsumerProgram, ConsumerDiscoveredProgramRoot>(
                &world.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = performed else {
            panic!("the original discovered source must publish");
        };
        let mut outputs = performed
            .start_required_outputs(
                &world.application,
                &request,
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap_or_else(|failure| panic!("original discovery: {:?}", failure.denial()));
        assert!(matches!(
            outputs
                .required_output_mut()
                .advance(&world.application, &request)
                .unwrap(),
            DiscoveredProgress::Progressed
        ));
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            1,
            "one root must return before its child discovery or another native root runs"
        );
        outputs
    };
    let settled = super::super::super::settle(|| {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        match outputs
            .required_output_mut()
            .advance(&world.application, &request)
            .unwrap()
        {
            DiscoveredProgress::Pending | DiscoveredProgress::Progressed => None,
            DiscoveredProgress::Settled(settled) => Some(settled),
        }
    });
    assert_eq!(
        settled
            .root_outputs()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert!(settled
        .root_outputs()
        .all(|(_, root)| root.producer_contacts_in_this_demand() == 1));
    assert_eq!(
        settled.output_count(),
        6,
        "all original root and dependent records remain unique"
    );
    assert_eq!(
        world
            .application
            .producer_contacts_on_this_thread_for_test(),
        4,
        "three distinct roots execute once and their shared dependent is reused"
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    for (key, expected) in [("remote-b", 42), ("sibling-b", 22), ("sibling-c", 31)] {
        assert_eq!(
            request
                .at(settled.observation())
                .query(PlanarOutputRead {
                    body_key: key.into(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(expected)
        );
    }
    request
        .at(&request.retain_current_read().unwrap())
        .require_current_discovered_program_outputs(&settled)
        .unwrap();
}
