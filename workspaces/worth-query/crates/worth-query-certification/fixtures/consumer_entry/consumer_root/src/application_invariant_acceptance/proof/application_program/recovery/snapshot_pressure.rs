use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryRequiredOutputPreparationDenial,
    WorthQueryRequiredOutputRecoveryPosture,
};
use worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial;
use worth_query_topology_entry::{
    PlanarOutputDemand, PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::super::{authentication, installation, seed::length};
use super::output_controls;
use crate::ConsumerSchema;

pub(in crate::application_invariant_acceptance::proof::application_program) fn snapshot_pressure_preserves_recoverable_source(
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
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_044)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the source commits before snapshot pressure");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("snapshot pressure requires one fresh committed source")
    };
    let receipt = performed.receipt().clone();
    drop(performed);

    let mut retained = Vec::new();
    let mut exhausted = false;
    for _ in 0..64 {
        match request.retain_read() {
            Ok(observation) => retained.push(observation),
            Err(error) => {
                assert!(
                    matches!(
                        error,
                        WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted
                    ),
                    "snapshot pressure failed for the wrong reason: {error:?}"
                );
                exhausted = true;
                break;
            }
        }
    }
    assert!(
        exhausted,
        "the finite World snapshot budget must be reached"
    );
    let denial = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &receipt,
            PlanarOutputDemand::new("anchor-b"),
            output_controls(),
        )
        .err()
        .expect("snapshot pressure denies recovery without consuming its source");
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryRequiredOutputRecoveryPosture::Retryable
    );
    assert!(matches!(
        denial,
        WorthQueryRequiredOutputPreparationDenial::Demand(
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Source(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ProductSelection(
                WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted
            )
            )
        )
    ), "recovery denial must expose snapshot capacity, not a proxy: {denial:?}");
    drop(retained);

    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &receipt,
            PlanarOutputDemand::new("anchor-b"),
            output_controls(),
        )
        .expect("released snapshot pressure permits the same source receipt");
    let mut settled = None;
    for _ in 0..64 {
        match recovered
            .advance(&request)
            .expect("recovered output advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(value) => {
                settled = Some(value);
                break;
            }
        }
    }
    let settled = settled.expect("recovered output settles within bounded advances");
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .expect("recovered output is readable")
            .rows()[0]
            .value,
        length(4)
    );
}
