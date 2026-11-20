//! Author and seal the complete original plan before its first journal write.
use super::{plan::*, preparation::*, refusal::*, resumption::RecomputedHomeProgramTransitionPlan};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::{
    application_contribution::WorthQueryBootstrapAdvancementPhase,
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
};
use worth_relational::facade::{
    durability::ReadmittedRelationalHomeProgramCatalogs, indexes::DerivedIndexMaintenanceBudget,
};
use worth_store_application::ApplicationHomeTransitionPlanningRequest as PlanningRequest;
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
    pub(super) origin: PlannedHomeProgramTransitionOrigin,
    pub(super) resources: DerivedIndexMaintenanceBudget,
}

/// Whether this plan starts an original intent or recomputes a recorded one.
pub(super) enum PlannedHomeProgramTransitionOrigin {
    /// Reserved original assignments; the intent is not yet published.
    Fresh(worth_store_application::ReservedApplicationHomeTransitionPlan),
    /// The recorded pending intent. Its original roots stay pinned until the
    /// binding, and Store compares this plan before admitting any step.
    Resumed {
        recomputed: RecomputedHomeProgramTransitionPlan,
        catalogs: ReadmittedRelationalHomeProgramCatalogs,
    },
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
        workflow_dispositions: Option<
            &worth_query_execution::facade::primary_graph::WorthQueryWorkflowDispositions,
        >,
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
        let Self {
            owners,
            binding,
            compilation,
        } = self;
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
            let (lens, catalogs) = match home
                .admit_home_program_transition_planning_request(source, &request)
                .map_err(Denial::PlanningRequest)?
            {
                PlanningRequest::Fresh(admitted) => (
                    before
                        .home_program_transition_source(
                            phase,
                            native,
                            source,
                            admitted,
                            &home.world().observation_port(),
                            &home.world().inspection_port(),
                        )
                        .map_err(Denial::Source)?,
                    None,
                ),
                PlanningRequest::Resume(required) => {
                    let (admitted, catalogs) = home
                        .readmit_home_program_transition_planning_root(
                            required,
                            &compilation,
                            source,
                            target.native_contracts(),
                            &request,
                        )
                        .map_err(Denial::PlanningRequest)?
                        .into_parts();
                    let lens = before
                        .home_program_transition_original_source(
                            phase,
                            native,
                            source,
                            catalogs.predecessor(),
                            admitted,
                        )
                        .map_err(Denial::Source)?;
                    (lens, Some(catalogs))
                }
            };
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
            if let Some(batch) = target
                .prepare_home_workflow_dispositions(
                    before,
                    &lens,
                    workflow_dispositions,
                    resources
                        .maximum_work_units
                        .expect("opening admitted finite transition resources"),
                )
                .map_err(Denial::Workflow)?
            {
                plan.steps[plan.target as usize].batches.push(batch);
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
            let recomputed = RecomputedHomeProgramTransitionPlan {
                digest: plan.digest(&bodies),
                steps: count,
                target: target
                    .native_contracts()
                    .target_meaning(&request, plan.target)
                    .map_err(Denial::Native)?,
                bodies,
            };
            let origin = match catalogs {
                None => PlannedHomeProgramTransitionOrigin::Fresh(
                    home.reserve_home_program_transition_plan(
                        source.clone(),
                        recomputed.digest,
                        recomputed.steps,
                        recomputed.target,
                        &recomputed.bodies,
                        &request,
                    )
                    .map_err(Denial::Plan)?,
                ),
                Some(catalogs) => PlannedHomeProgramTransitionOrigin::Resumed {
                    recomputed,
                    catalogs,
                },
            };
            Ok((bridge, plan, origin))
        })();
        // A resumed plan's catalogs now hold the planning root.
        drop(compilation);
        let (bridge, plan, origin) = match prepared {
            Ok(parts) => parts,
            Err(cause) => return Err(owners.refused(cause)),
        };
        Ok(PlannedHomeProgramTransition {
            owners,
            binding,
            bridge,
            plan,
            origin,
            resources,
        })
    }
}
