//! Native settlement returns original custody before independent dependent discovery.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramOutputProgress as Progress,
    WorthQueryApplicationProgramOutputSettlement,
};
use worth_query_host::facade::primary_graph::WorthQueryExecutionObservation;
use worth_query_topology_entry::{
    PlanarAlternateFinalToSummaryConnection, PlanarFinalToSummaryConnection, PlanarOutputRead,
    PlanarOutputToAlternateFinalConnection, PlanarOutputToFinalConnection,
};

mod discovered;
mod no_source;

#[test]
fn performed_tree_yields_at_root_and_child_settlement_before_discovery() {
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
        let performed = request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(2),
            })
            .expect_source(source)
            .idempotency(&10_721)
            .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
                &world.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
            panic!("the original source must publish");
        };
        performed
            .start_required_outputs(
                &world.application,
                &request,
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap_or_else(|failure| panic!("original outputs: {:?}", failure.denial()))
    };
    let original_source = outputs
        .required_output_mut()
        .source_observation()
        .selected_commit()
        .clone();
    super::super::settle(|| {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let tree = outputs.required_output_mut();
        let progress = tree.advance(&world.application, &request).unwrap();
        assert!(
            matches!(progress, Progress::Progressed),
            "a runnable native tree must return before dependent discovery"
        );
        tree.settled_root_observation().map(|_| ())
    });
    assert_eq!(
        world
            .application
            .producer_contacts_on_this_thread_for_test(),
        1,
        "root settlement must not execute a dependent in the same advance"
    );
    assert_eq!(
        outputs
            .required_output_mut()
            .source_observation()
            .selected_commit(),
        &original_source
    );

    super::super::settle(|| {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let progress = outputs
            .required_output_mut()
            .advance(&world.application, &request)
            .unwrap();
        assert!(matches!(progress, Progress::Progressed));
        (world
            .application
            .producer_contacts_on_this_thread_for_test()
            > 1)
        .then_some(())
    });
    assert_eq!(
        world
            .application
            .producer_contacts_on_this_thread_for_test(),
        2,
        "child settlement must return through the tuple before its descendant or sibling executes"
    );

    let settlement = super::super::settle(|| {
        let scope = authentication::request_scope();
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        super::super::settled(
            outputs
                .required_output_mut()
                .advance(&world.application, &request)
                .unwrap(),
        )
    });
    assert_complete_original_tree(&settlement);
    assert_eq!(
        world
            .application
            .producer_contacts_on_this_thread_for_test(),
        5,
        "the original root, both siblings and both descendants each execute once"
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
    for (key, value) in [
        ("final:anchor-a", 4),
        ("final:final:anchor-a", 6),
        ("final:anchor-b", 3),
        ("final:final:anchor-b", 5),
    ] {
        assert_eq!(
            request
                .at(settlement.observation())
                .query(PlanarOutputRead {
                    body_key: key.into(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(value)
        );
    }
    request
        .at(&request.retain_current_read().unwrap())
        .require_current_program_output(&settlement)
        .unwrap();
}

fn assert_complete_original_tree<Query>(
    settlement: &WorthQueryApplicationProgramOutputSettlement<Query>,
) {
    assert_eq!(settlement.root_producer_contacts_in_this_demand(), 1);
    assert_eq!(
        settlement.output_count(),
        4,
        "no output record is lost or duplicated on resume"
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-a"]
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToAlternateFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["anchor-b"]
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarFinalToSummaryConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["final:anchor-a"]
    );
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["final:anchor-b"]
    );
}
