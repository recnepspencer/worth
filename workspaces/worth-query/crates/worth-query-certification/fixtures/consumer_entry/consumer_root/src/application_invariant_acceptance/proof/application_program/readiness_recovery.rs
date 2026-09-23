use std::sync::atomic::Ordering;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationRequestExt,
    WorthQueryRequiredOutputPreparationDenial,
};
use worth_query_topology_entry::{
    PlanarEdit, PlanarMutation, PlanarOutputDemand, PlanarOutputRead, PlanarRead,
};
use worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial;

use super::super::super::{authentication, installation, seed::length};
use super::super::adjust;
use super::lifecycle::{controls, perform};
use crate::ConsumerSchema;

mod already_committed;
mod dependency_aba;
mod preserved_noop;
mod published_capacity;
mod ready_read;
pub(super) use already_committed::already_committed_replace_reuses_readiness;
pub(super) use dependency_aba::complete_dependency_aba_advances_the_live_demand;
pub(super) use preserved_noop::preserved_noop_output_completes_readiness_without_a_signal_successor;
pub(super) use published_capacity::published_outputs_hold_no_hidden_read_lease;
pub(super) use ready_read::ready_read_capacity_preserves_completion;

pub(super) fn program_demand_rejects_a_foreign_request_and_releases_on_drop(
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
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the program root admits its declared output");
    match demand.advance(&foreign_request) {
        Err(WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch,
        )) => {}
        Err(WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Source(
                worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::PrincipalResolution(_),
            ),
        )) => {}
        Err(other) => panic!("unexpected foreign request denial: {other:?}"),
        Ok(_) => panic!("the foreign request unexpectedly advanced the program"),
    }
    drop(demand);
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
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the first program interest admits");
    let mut peer = request
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the peer program interest admits");
    world
        .producer_authorization_denials
        .store(1, Ordering::SeqCst);
    let denial = (0..64)
        .find_map(|_| match denied.advance(&request) {
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("the injected producer denial must prevent the first settlement")
            }
            Err(denial) => Some(denial),
        })
        .expect("the producer denial must surface within bounded advances");
    assert!(matches!(
        denial,
        WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Demand(denial)
        ) if denial.kind()
            == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProducerUnavailable
    ));
    let settled = (0..64).find_map(|_| match peer.advance(&request).expect("the peer resumes") {
        WorthQueryApplicationProgramOutputProgress::Pending => None,
        WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
    });
    assert!(settled.is_some(), "the peer must settle within bounded advances");
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
    assert!(matches!(
        output.required_output_mut().advance(&request).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));
    world
        .application
        .fail_next_output_readiness_evaluation_for_test();
    let failure = (0..64)
        .find_map(|_| match output.required_output_mut().advance(&request) {
            Err(failure) => Some(failure),
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("readiness fault must deny before output settlement")
            }
        })
        .expect("the injected readiness evaluation must fail after output publication");
    assert_eq!(
        failure.recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Retryable,
    );
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
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
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
pub(super) fn readiness_snapshot_pressure_keeps_published_output_recoverable(
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
    let mut output = perform(&request, &world.application, "anchor-c", 2, 10_023);
    let source_receipt = output.receipt().clone();
    assert!(matches!(
        output.required_output_mut().advance(&request).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));
    assert!(matches!(
        output.required_output_mut().advance(&request).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));

    world
        .application
        .press_next_readiness_with_world_snapshots_for_test();
    let mut failure = None;
    for _ in 0..64 {
        match output.required_output_mut().advance(&request) {
            Err(denial) => {
                failure = Some(denial);
                break;
            }
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => {}
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("readiness selection must encounter real World capacity")
            }
        }
    }
    let failure =
        failure.expect("readiness selection reaches World capacity within bounded advances");
    assert_eq!(
        failure.recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Retryable,
        "{failure:?}"
    );
    assert!(
        matches!(
            &failure,
            worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(
                    denial
                )
            ) if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProductSelection(
                WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted
            )
        ),
        "readiness denial must retain its typed capacity cause: {failure:?}"
    );
    drop(output);

    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &source_receipt,
            PlanarOutputDemand::new("anchor-c"),
            controls(),
        )
        .expect("published readiness remains in Query custody after capacity is released");
    let mut settled = None;
    for _ in 0..64 {
        match recovered.advance(&request).expect("readiness resumes") {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(value) => {
                settled = Some(value);
                break;
            }
        }
    }
    let settled = settled.expect("readiness must settle within bounded advances");
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-c".to_owned(),
            })
            .execute()
            .expect("the published result is exact after recovery")
            .rows()[0]
            .value,
        length(4)
    );
}
