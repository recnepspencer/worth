use std::sync::atomic::Ordering;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationPerformedMutationOutcome,
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
    WorthQueryRequiredOutputPreparationDenial,
};
use worth_query_topology_entry::{
    PlanarOutputDemand, PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use super::lifecycle::{controls, perform};
use crate::ConsumerSchema;

pub(super) fn program_demand_closes_and_rejects_a_foreign_request(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let other = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let foreign_request = other.application.request(&principal, &scope);
    let mut demand = request
        .start_program_outputs(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the program root admits its declared output");
    assert!(matches!(
        demand.advance(&foreign_request),
        Err(WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch
        ))
    ));
    demand.close();
    demand.close();
    assert!(matches!(
        demand.advance(&request),
        Err(WorthQueryRequiredOutputPreparationDenial::Closed)
    ));
}

pub(super) fn denied_program_producer_releases_the_shared_claim(
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
    let mut denied = request
        .start_program_outputs(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the first program interest admits");
    let mut peer = request
        .start_program_outputs(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the peer program interest admits");
    world
        .producer_authorization_denials
        .store(1, Ordering::SeqCst);
    let denial = loop {
        match denied.advance(&request) {
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => {}
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("the injected producer denial must prevent the first settlement")
            }
            Err(denial) => break denial,
        }
    };
    assert!(matches!(
        denial,
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(denial)
        )
            if denial.kind()
                == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProducerUnavailable
    ));
    loop {
        match peer
            .advance(&request)
            .expect("the peer resumes the released claim")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
        }
    }
}

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
        .start_program_outputs(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the initial output demand starts");
    loop {
        match initial
            .advance(&request)
            .expect("the initial output settles")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
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
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(5),
        })
        .expect_source(related.observed_sources()[0].clone())
        .idempotency(&10_021)
        .execute_performed(&world.application)
        .expect("the related dependency changes");
    assert!(matches!(
        outcome,
        WorthQueryApplicationPerformedMutationOutcome::Performed(_)
    ));

    let mut preserved = request
        .start_program_outputs(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the drifted source selects its preserve producer");
    let settlement = loop {
        match preserved
            .advance(&request)
            .expect("an exact preserved no-op still completes readiness")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settlement) => break settlement,
        }
    };
    let delivery = settlement
        .root()
        .readiness_delivery()
        .expect("preserved output readiness carries delivery evidence");
    assert!(
        !delivery.has_conditional_successor(),
        "the proof must exercise the no-successor preserve path"
    );
    assert_eq!(
        request
            .at(settlement.root().observation())
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
