use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryRequiredOutputRecoveryPosture,
};
use worth_query_topology_entry::{PlanarOutputDemand, PlanarRead, PlanarSourceAdjustment};

use super::super::super::super::{authentication, installation, seed::length};
use super::super::lifecycle::{controls, perform};
use crate::ConsumerSchema;

pub(in crate::application_invariant_acceptance::proof::application_program) fn superseded_completion_is_terminal(
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
    let mut output = perform(&request, &world.application, "anchor-c", 2, 10_045);
    let receipt = output.receipt().clone();
    let mut settled = false;
    for _ in 0..64 {
        match output
            .required_output_mut()
            .advance(&request)
            .expect("output advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => {
                settled = true;
                break;
            }
        }
    }
    assert!(
        settled,
        "the prior output must settle before it is superseded"
    );
    drop(output);

    let current = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .expect("the current source remains readable");
    let replacement = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-c".to_owned(),
            replacement_y: length(6),
        })
        .expect_source(current.observed_sources()[0].clone())
        .idempotency(&10_046)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the new source intent commits through the installed program");
    assert!(matches!(
        &replacement,
        WorthQueryApplicationPerformedMutationOutcome::Performed(_)
    ));
    drop(replacement);

    let denial = request
        .recover_required_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &receipt,
            PlanarOutputDemand::new("anchor-c"),
            controls(),
        )
        .err()
        .expect("a superseded completion cannot remain recoverable");
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryRequiredOutputRecoveryPosture::Terminal,
        "{denial:?}"
    );
    assert!(
        matches!(
            denial,
            worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(ref cause)
            ) if cause.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
        ),
        "the superseded source must lose output custody before recovery: {denial:?}"
    );
}
