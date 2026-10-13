//! The required wave names the demand's stop at its real Product/native selection.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, installed_program_support, rostered_program_revision,
    seed_program_activation,
};
#[test]
fn required_wave_selection_answers_its_stopped_request_by_value() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        world.application.installed_schema(),
    ));
    world
        .application
        .with_host_advancement(|phase| {
            seed_program_activation(
                &phase.borrowed_request_for_test(),
                &world,
                &rostered_program_revision(),
            )
        })
        .unwrap();
    crate::test_read_stops::every_consult(|execution| {
            let mut admission = world.application.primary_provider.graph.source_owner.invalidation_owner.edit_admission_within(std::num::NonZeroUsize::new(100_000).unwrap());
            select_required_basis(execution, &world.application, world.application.current_world(), &mut admission).map(|_| ()).map_err(|denial| denial.kind())
        }, WorthQueryOutputDemandDenialKind::Cancelled, WorthQueryOutputDemandDenialKind::SourceQueryExecution(crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None,
        })));
}
