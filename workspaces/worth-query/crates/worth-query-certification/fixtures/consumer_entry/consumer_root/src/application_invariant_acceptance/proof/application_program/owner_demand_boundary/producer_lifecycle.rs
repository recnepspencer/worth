use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_contribution::WorthQueryProducerLifecyclePosture,
    application_entry::{
        WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
        WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
    },
    primary_graph::WorthQueryOutputDemandRecoveryPosture,
};
use worth_query_topology_entry::PlanarSourceAdjustment;

use super::super::super::super::seed::length;
use super::*;

pub(super) fn live_output_and_stale_head_selection(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign_schema);
    let application = &world.application;
    let scope = authentication::request_scope();
    let principal = authenticate(application, &scope);
    let request = application.request(&principal, &scope);
    publish_initial_output(&request, application, &principal, &scope);

    let initial = source(application, &principal, &scope, "anchor-a");
    assert_eq!(
        selected_lifecycle(application, &initial),
        WorthQueryProducerLifecyclePosture::Initial,
    );

    let exact = source(application, &principal, &scope, "anchor-a");
    assert_eq!(
        selected_lifecycle(application, &exact),
        WorthQueryProducerLifecyclePosture::Initial,
        "an unchanged live source retains its exact initial producer",
    );
    let exhausted = application
        .select_output_producer::<PlanarOutputFamily>(&exact.observed_sources()[0], "planar", 1)
        .expect_err("the live-output lookup must fit its declared work budget");
    assert_eq!(
        exhausted.kind(),
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
    );
    assert_eq!(
        exhausted.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable,
    );
    let stale = source(application, &principal, &scope, "anchor-a");
    adjust(
        &request,
        application,
        &principal,
        &scope,
        "anchor-b",
        9,
        10_700,
    );
    let denied = application
        .select_output_producer::<PlanarOutputFamily>(&stale.observed_sources()[0], "planar", 4_096)
        .expect_err("a moved head cannot grant output lifecycle authority from an old source");
    assert_eq!(denied.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    assert_eq!(
        denied.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable,
    );
    let refreshed = source(application, &principal, &scope, "anchor-a");
    assert_eq!(
        selected_lifecycle(application, &refreshed),
        WorthQueryProducerLifecyclePosture::Preserve,
        "a fresh source observation may preserve the live output after product publication",
    );

    adjust(
        &request,
        application,
        &principal,
        &scope,
        "anchor-a",
        9,
        10_701,
    );
    let revised = source(application, &principal, &scope, "anchor-a");
    assert_eq!(
        selected_lifecycle(application, &revised),
        WorthQueryProducerLifecyclePosture::Preserve,
        "a changed source with a live prior output must preserve it",
    );
}

fn publish_initial_output(
    request: &super::super::super::Request<'_>,
    application: &super::super::super::ProgramApplication,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    scope: &WorthQueryRequestScope,
) {
    let observed = source(application, principal, scope, "anchor-a").observed_sources()[0].clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(observed)
        .idempotency(&10_699_u64)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(application)
        .expect("the initial source reaches the program entry");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the initial source must retain performed delivery")
    };
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let mut outputs = performed
        .start_required_outputs(request, controls)
        .unwrap_or_else(|failure| {
            panic!(
                "the program binds its initial required output: {:?}",
                failure.denial()
            )
        });
    for _ in 0..512 {
        if matches!(
            outputs
                .required_output_mut()
                .advance(request)
                .expect("the initial output advances"),
            WorthQueryApplicationProgramOutputProgress::Settled(_),
        ) {
            return;
        }
        std::thread::yield_now();
    }
    panic!("the initial program output did not settle within its synchronous bound");
}

fn selected_lifecycle(
    application: &super::super::super::ProgramApplication,
    disclosed: &WorthQueryApplicationOutputDemandSource<PlanarQuery, PlanarReadResult>,
) -> WorthQueryProducerLifecyclePosture {
    application
        .select_output_producer::<PlanarOutputFamily>(
            &disclosed.observed_sources()[0],
            "planar",
            4_096,
        )
        .expect("the installed producer selects for the real source")
        .applicability()
        .lifecycle()
}

fn adjust(
    request: &super::super::super::Request<'_>,
    application: &super::super::super::ProgramApplication,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    scope: &WorthQueryRequestScope,
    key: &str,
    y: u64,
    command: u64,
) {
    let observed = source(application, principal, scope, key).observed_sources()[0].clone();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: key.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed)
        .idempotency(&command)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(application)
        .expect("the independent source adjustment commits");
}
