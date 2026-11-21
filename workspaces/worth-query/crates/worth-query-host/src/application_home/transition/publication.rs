//! Execute each original positioned World open once through Native's compiler.
use super::super::runtime::DirectoryHome;
use super::{
    refusal::WorthQueryHomeProgramTransitionDenial as Denial,
    step_execution::{self, HomeProgramTransitionSteps},
};
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::{
    PreparedExistingHomeSignalProgram, PreparedHomeProgramTransitionTarget,
};
use worth_runtime_world::facade::*;

pub(super) fn publish<
    A: ApplicationSchemaComposition,
    AP,
    Schema: ApplicationSchemaComposition,
    Program,
>(
    home: &mut DirectoryHome,
    target: &PreparedHomeProgramTransitionTarget<Schema, Program>,
    predecessor: &PreparedExistingHomeSignalProgram<A, AP>,
    mut steps: HomeProgramTransitionSteps,
    budget: worth_relational::facade::indexes::DerivedIndexMaintenanceBudget,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let cancellation = RuntimeWorldCancellationSource::new();
    let count = steps.count()?;
    for position in 0..count {
        let source = home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .map_err(Denial::Observation)?;
        let program_source = target
            .admit_home_transition_program_source(
                predecessor,
                home.relational().runtime(),
                &source,
                &home.branch_program_source_owner(),
                request,
            )
            .map_err(Denial::Author)?;
        let mut grant = home
            .admit_home_program_transition_step(position, &source, request)
            .map_err(Denial::Intent)?;
        let reservation = grant.take_reservation().ok_or(Denial::InvalidPlan)?;
        let pending = home
            .world()
            .publication_port()
            .prepare_home_transition_step(
                request,
                source.clone(),
                program_source,
                steps.change(position),
                grant.plan(),
                grant.original(),
                reservation,
                grant.admission(),
                &cancellation.token(),
                None,
            )
            .map_err(|(reservation, cause)| Denial::OpenPreparation { reservation, cause })?;
        let opened = step_execution::opened(pending.advance(request), request)?;
        let performed = step_execution::execute(
            home,
            target,
            &mut steps,
            position,
            &source,
            &grant,
            opened,
            budget,
            request,
            &cancellation,
        )?;
        home.accept_home_program_transition_step(performed, request)
            .map_err(Denial::Completion)?;
    }
    Ok(())
}
