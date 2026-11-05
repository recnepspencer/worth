//! Assign a Home journal position to World-issued original attempt reservations.
use crate::home_context::HomeContext;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_runtime_world::facade::{
    ProductBranchObservation, ReservedWorldHomeTransitionAssignments, RuntimeWorldPublicationPort,
};
use worth_store::physical_runtime::{
    OrderedJournalDenial, OrderedJournalName, OrderedJournalReadFailure,
};

#[derive(Debug)]
pub enum ApplicationHomeTransitionPlanningDenial {
    Execution(worth_execution::WorkCeilingDenial),
    Journal(OrderedJournalDenial),
    Read(OrderedJournalReadFailure),
    PositionExhausted,
    SourceScope,
    Intent(super::ApplicationHomeTransitionIntentDenial),
    Native(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    World(worth_runtime_world::facade::WorldHomeTransitionReservationRefusal),
}

/// The actual World reservation and the original request's description backing.
pub struct ReservedApplicationHomeTransitionPlan {
    pub(super) world: ReservedWorldHomeTransitionAssignments,
    pub(super) native: worth_relational::facade::branch::RecordedRelationalComponentReference,
    pub(super) _description_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ReservedApplicationHomeTransitionPlan {
    pub fn description(&self) -> &HomeProgramTransitionPlanDescription {
        self.world.description()
    }
    pub fn assignments(&self) -> &[CompositeAttemptAssociation] {
        self.world.assignments()
    }
}

pub(crate) fn reserve<D, I, E, Ctx, T>(
    context: &HomeContext,
    publication: RuntimeWorldPublicationPort<D, I, E, Ctx, T>,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    expected: ProductBranchObservation,
    digest: HomeProgramTransitionPlanDigest,
    steps: u32,
    target: HomeProgramTransitionTargetMeaning,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ReservedApplicationHomeTransitionPlan, ApplicationHomeTransitionPlanningDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    crate::request_stage::prepare(
        request,
        || {
            reserve_prepared(
                context,
                publication,
                native,
                expected,
                digest,
                steps,
                target,
                request,
            )
        },
        ApplicationHomeTransitionPlanningDenial::Execution,
    )
}

fn reserve_prepared<D, I, E, Ctx, T>(
    context: &HomeContext,
    publication: RuntimeWorldPublicationPort<D, I, E, Ctx, T>,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    expected: ProductBranchObservation,
    digest: HomeProgramTransitionPlanDigest,
    steps: u32,
    target: HomeProgramTransitionTargetMeaning,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ReservedApplicationHomeTransitionPlan, ApplicationHomeTransitionPlanningDenial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    use ApplicationHomeTransitionPlanningDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    context
        .transition
        .admission
        .require_fresh_plan(&context.descriptor.binding)
        .map_err(Denial::Intent)?;
    let journal = context
        .serving
        .ordered_journal(
            OrderedJournalName::new("home.transitions")
                .expect("the closed Home transition journal"),
        )
        .map_err(Denial::Journal)?;
    let next = journal
        .head()
        .map_err(Denial::Read)?
        .map_or(Some(1), |head| head.position().checked_add(1))
        .and_then(std::num::NonZeroU64::new)
        .ok_or(Denial::PositionExhausted)?;
    let owner = expected
        .owner_identity()
        .durable()
        .ok_or(Denial::SourceScope)?;
    if owner.home() != context.descriptor.identity {
        return Err(Denial::SourceScope);
    }
    let description_memory = request
        .reserve_retained_memory(expected.branch_identity().name().as_str().len() as u64)
        .map_err(Denial::Execution)?;
    let branch = DurableProductBranchIdentity::from_recorded(
        owner,
        expected.branch_identity().name().as_str().to_owned(),
    )
    .map_err(|_| Denial::SourceScope)?;
    let predecessor = DurableProductBranchReference::from_recorded(
        branch,
        DurableProductBranchIncarnation::from_recorded(
            owner,
            expected.lifecycle_incarnation().ordinal(),
        ),
        DurableProductReferenceGeneration::from_recorded(expected.reference_generation().get()),
        DurableCompositeCommitIdentity::from_recorded(owner, expected.selected_commit().ordinal()),
    )
    .map_err(|_| Denial::SourceScope)?;
    let description = HomeProgramTransitionPlanDescription::from_recorded(
        HomeProgramTransitionIdentity::from_recorded(context.descriptor.identity, next),
        predecessor,
        digest,
        steps,
        target,
    )
    .map_err(|_| Denial::SourceScope)?;
    let proof = context.transition.admit_plan(description.commitment());
    let native = native
        .owner_component_services()
        .basis_port()
        .export_recorded_current_component(request, expected.basis().relational_basis())
        .map_err(Denial::Native)?;
    let world = publication
        .reserve_home_transition_assignments(request, expected, description, proof)
        .map_err(Denial::World)?;
    Ok(ReservedApplicationHomeTransitionPlan {
        world,
        native,
        _description_memory: description_memory,
    })
}
