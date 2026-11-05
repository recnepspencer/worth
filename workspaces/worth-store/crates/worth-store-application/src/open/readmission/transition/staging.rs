//! Stage only the actual predecessor selected by this Home's attached owners.
use super::*;
use worth_foundational::facade::ApplicationHomeBinding;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

pub(crate) fn stage(
    context: &HomeContext,
    native: &RelationalRuntime,
    world: RuntimeWorldOwnerIdentity,
    reserved: ReservedApplicationHomeTransitionPlan,
    successor: worth_proof::AdmittedApplicationHomeBinding<
        worth_query_installation::facade::WorthQueryHomeBindingAdmission,
    >,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeTransitionIntentDenial> {
    crate::request_stage::prepare(
        request,
        || stage_prepared(context, native, world, reserved, successor, request),
        ApplicationHomeTransitionIntentDenial::Execution,
    )
}

fn stage_prepared(
    context: &HomeContext,
    native: &RelationalRuntime,
    world: RuntimeWorldOwnerIdentity,
    reserved: ReservedApplicationHomeTransitionPlan,
    successor: worth_proof::AdmittedApplicationHomeBinding<
        worth_query_installation::facade::WorthQueryHomeBindingAdmission,
    >,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), ApplicationHomeTransitionIntentDenial> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    if reserved.description().identity().home() != context.descriptor.identity
        || reserved.world.predecessor().owner_identity() != world
    {
        return Err(Denial::ForeignHome);
    }
    let successor = successor.consume(|admitted| admitted.into_description());
    context
        .transition
        .admission
        .require_fresh_successor(&context.descriptor.binding, &successor)?;
    let actual = native
        .owner_component_services()
        .basis_port()
        .export_recorded_current_component(
            request,
            reserved.world.predecessor().basis().relational_basis(),
        )
        .map_err(Denial::Native)?;
    if actual != reserved.native {
        return Err(Denial::ForeignPredecessor);
    }
    require_original(context, &reserved, &successor)?;
    let publication = HomeIntentPublication::prepare(
        &context.serving,
        reserved,
        &context.descriptor.binding,
        &successor,
        request,
    )?;
    context.transition.install_publication(publication)
}
fn require_original(
    context: &HomeContext,
    reserved: &ReservedApplicationHomeTransitionPlan,
    successor: &ApplicationHomeBinding,
) -> Result<(), ApplicationHomeTransitionIntentDenial> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    let Some(original) = context.transition.publication() else {
        return Ok(());
    };
    original.intent.require_plan(
        successor,
        reserved.description().digest(),
        reserved.description().steps(),
        reserved.description().target_meaning(),
    )?;
    if original.intent.plan() != reserved.description()
        || original.intent.native_predecessor() != &reserved.native
        || original.intent.predecessor() != &context.descriptor.binding
    {
        return Err(Denial::SourceMismatch);
    }
    Ok(())
}
pub(crate) fn advance(
    context: &HomeContext,
    request: &ExecutionRequest<'_, '_>,
) -> Result<ApplicationHomeTransitionIntentOutcome, ApplicationHomeTransitionIntentDenial> {
    context
        .transition
        .publication()
        .ok_or(ApplicationHomeTransitionIntentDenial::SourceMismatch)?
        .advance(&context.serving, request)
}
