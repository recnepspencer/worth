use super::*;
use crate::domain_computation::primary_graph::{
    tests::application_query::observed_account_source, WorthQueryProducerApplicability,
};
use crate::domain_computation::WorthQueryProviderSessionDenialKind as Kind;

#[test]
fn scheduling_progression_preserves_admission_stops_at_every_consult() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(
        crate::domain_computation::primary_graph::tests::fixture::installed_program_support(
            world.application.installed_schema(),
        ),
    );
    world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::fixture::seed_program_activation(
            &phase.borrowed_request_for_test(), &world,
            &crate::domain_computation::primary_graph::tests::fixture::rostered_program_revision(),
        );
        })
        .unwrap();
    let source = observed_account_source(&world);
    let (_, route) = installation();
    world
        .application
        .output_producer_routes
        .insert("scheduling-busy".to_owned(), route);
    let selected = WorthQuerySelectedApplicationProducer {
        identity: "scheduling-busy".to_owned(),
        applicability: WorthQueryProducerApplicability::new(
            "account",
            crate::domain_computation::primary_graph::WorthQueryProducerLifecyclePosture::Initial,
        ),
        exact_retained_output: false,

        retained_idempotency_key: None,
        retained_output_binding: None,
        reuses_live_output_only: false,
    };
    // The installed route is foreign to this fixture's bridge. Its ordinary
    // refusal is the baseline; the actual scheduling admission read must still
    // propagate every stop before the bridge is contacted.
    world.application.with_host_advancement(|phase| {
        crate::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(),
            |execution| world.application.schedule_selected_output_producer(
                execution, &phase, &selected, world.application.current_world(), &source, None,
             ).map(|_| ()).map_err(|denial| denial.kind()).or_else(|kind| {
                if matches!(&kind, WorthQueryOutputDemandDenialKind::BridgeConditional(native)
                    if matches!(native.as_ref(), worth_runtime_bridge::facade::BridgeConditionalDenialKind::StaleLowering)) {
                    Ok(())
                } else {
                    Err(kind)
                }
            }),
            WorthQueryOutputDemandDenialKind::Cancelled,
            WorthQueryOutputDemandDenialKind::SourceQueryExecution(
                crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(Kind::ExecutionResource {
                    denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                    partition_identity: None, policy_ancestor: None,
                }),
            ),
        );
    }).unwrap();
}
