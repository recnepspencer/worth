//! Author and seal the complete original plan before its first journal write.
use super::{plan::*, preparation::*, refusal::*};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_contribution::WorthQueryBootstrapAdvancementPhase,
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
};
use worth_relational::facade::indexes::DerivedIndexMaintenanceBudget;
use WorthQueryHomeProgramTransitionDenial as Denial;

pub(super) struct PlannedHomeProgramTransition<
    A: ApplicationSchemaComposition,
    AP,
    B: ApplicationSchemaComposition,
    BP,
> {
    pub(super) owners: Box<ReadmittedHomeProgramTransition<A, AP, B, BP>>,
    pub(super) binding: SuccessorBinding,
    pub(super) bridge: worth_runtime_bridge::facade::RuntimeBridge,
    pub(super) plan: WorthQueryHomeProgramTransitionPlan,
    pub(super) reserved: worth_store_application::ReservedApplicationHomeTransitionPlan,
    pub(super) resources: DerivedIndexMaintenanceBudget,
}
impl<A: ApplicationSchemaComposition, AP, B: ApplicationSchemaComposition, BP>
    PreparedHomeProgramTransition<A, AP, B, BP>
{
    #[inline(never)]
    pub(super) fn plan<Author>(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        author: Author,
        resources: DerivedIndexMaintenanceBudget,
    ) -> Result<
        PlannedHomeProgramTransition<A, AP, B, BP>,
        WorthQueryHomeProgramTransitionOpenRefused<A, AP, B, BP>,
    >
    where
        Author: FnOnce(
            &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
            &WorthQueryHomeProgramTransitionTarget<'_>,
        ) -> Result<
            WorthQueryHomeProgramTransitionPlan,
            worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
        >,
    {
        let Self { owners, binding } = self;
        let request = phase.execution_request();
        let before = &owners.before;
        let target = &owners.target;
        let home = &owners.home;
        let source = &owners.source;
        let prepared = (|| {
            let native = home.relational().runtime();
            let (_, bridge, signal) = home
                .query_composition_parts()
                .map_err(Denial::Composition)?;
            target
                .require_relational_programme_transition()
                .map_err(Denial::Author)?;
            signal
                .preflight_relational_programme_transition()
                .map_err(|cause| Denial::BridgeProgramme(Box::new(cause)))?;
            let bridge = target
                .prepare_home_program_bridge(&bridge)
                .map_err(Denial::Author)?;
            let lens = before
                .home_program_transition_source(phase, native, source.basis().relational_basis())
                .map_err(Denial::Author)?;
            let mut plan = author(&lens, &target.authoring_target()).map_err(Denial::Author)?;
            let count = u32::try_from(plan.steps.len()).map_err(|_| Denial::InvalidPlan)?;
            if count == 0 || plan.target >= count {
                return Err(Denial::InvalidPlan);
            }
            for (position, step) in plan.steps.iter().enumerate() {
                lens.admit_authored_body(
                    &target.authoring_target(),
                    plan.role(position as u32),
                    &step.batches,
                )
                .map_err(Denial::AuthoredBody)?;
            }
            plan.steps[plan.target as usize].batches.push(
                target
                    .bind_home_program_transition_activation(before, &lens)
                    .map_err(Denial::Author)?,
            );
            let bodies = plan
                .steps
                .iter()
                .enumerate()
                .map(|(position, step)| {
                    worth_relational::facade::runtime::home_program_transition_body_digest(
                        &request,
                        &plan.intent(position as u32),
                        &step.batches,
                    )
                    .map_err(Denial::Body)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let reserved = home
                .reserve_home_program_transition_plan(
                    source.clone(),
                    plan.digest(&bodies),
                    count,
                    target
                        .native_contracts()
                        .target_meaning(&request, plan.target)
                        .map_err(Denial::Native)?,
                    &bodies,
                    &request,
                )
                .map_err(Denial::Plan)?;
            Ok((bridge, plan, reserved))
        })();
        let (bridge, plan, reserved) = match prepared {
            Ok(parts) => parts,
            Err(cause) => return Err(owners.refused(cause)),
        };
        Ok(PlannedHomeProgramTransition {
            owners,
            binding,
            bridge,
            plan,
            reserved,
            resources,
        })
    }
}
