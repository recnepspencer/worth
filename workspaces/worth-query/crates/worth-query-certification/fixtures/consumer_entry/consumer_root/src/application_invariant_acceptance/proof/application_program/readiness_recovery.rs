use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
};
use worth_query_topology_entry::{PlanarMutation, PlanarOutputDemand, PlanarOutputRead, PlanarRead};

use super::super::super::{authentication, installation, seed::length};
use super::lifecycle::{controls, perform};
use crate::ConsumerSchema;

pub(super) fn readiness_failure_recovers_exact_pending_output(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let mut output = perform(&request, &world.application, "anchor-c", 2, 10_020);
    let source_receipt = output.receipt().clone();
    assert!(matches!(
        output.required_output_mut().advance(&request).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));
    world
        .application
        .fail_next_output_readiness_evaluation_for_test();
    let failure = match output.required_output_mut().advance(&request) {
        Err(failure) => failure,
        Ok(_) => panic!("the injected readiness evaluation must fail after output publication"),
    };
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = failure
    else {
        panic!("readiness failure must preserve the exact output-demand cause")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::SchedulingDeferred
    );
    drop(output);

    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram>(
            &world.application,
            &source_receipt,
            PlanarOutputDemand::new("anchor-c"),
            controls(),
        )
        .expect("owner-held readiness custody is recoverable");
    let settled = loop {
        match recovered.advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-c".to_owned(),
            })
            .execute()
            .expect("the recovered transitive output remains exact")
            .rows()[0]
            .value,
        length(4)
    );
}

pub(super) fn preserved_noop_output_completes_readiness_without_a_signal_successor(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let mut initial = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start()
        .expect("the initial output demand starts");
    loop {
        match initial.advance(&request).expect("the initial output settles") {
            WorthQueryApplicationOutputDemandProgress::Pending => {}
            WorthQueryApplicationOutputDemandProgress::Settled(_) => break,
        }
    }
    initial.close();

    let related = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the related source is observed");
    let outcome = request
        .mutate(PlanarMutation {
            scope_key: "anchor-b".to_owned(),
            operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                worth_query_consumer_values::PlanarAdjustment {
                    body_key: "anchor-b".to_owned(),
                    replacement_y: length(5),
                },
            ]),
            validator_work: 4_096,
        })
        .expect_source(related.observed_sources()[0].clone())
        .idempotency(&10_021)
        .execute()
        .expect("the related dependency changes");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));

    let mut preserved = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start()
        .expect("the drifted source selects its preserve producer");
    let settlement = loop {
        match preserved
            .advance(&request)
            .expect("an exact preserved no-op still completes readiness")
        {
            WorthQueryApplicationOutputDemandProgress::Pending => {}
            WorthQueryApplicationOutputDemandProgress::Settled(settlement) => break settlement,
        }
    };
    let delivery = settlement
        .readiness_delivery()
        .expect("preserved output readiness carries delivery evidence");
    assert!(
        !delivery.has_conditional_successor(),
        "the proof must exercise the no-successor preserve path"
    );
    assert_eq!(
        request
            .at(settlement.observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the preserved output remains readable")
            .rows()[0]
            .value,
        length(2)
    );
    preserved.close();
}
