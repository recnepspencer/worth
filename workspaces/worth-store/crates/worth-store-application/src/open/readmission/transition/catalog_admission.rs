//! Only actual committed intent DATA enters Native catalog readmission.
use super::*;
use worth_foundational::facade::HomeProgramTransitionStepAssociation;
use worth_proof::AdmittedHomeProgramTransitionCatalog;
use worth_relational::facade::runtime::RelationalHomeProgramCatalogReadmissionSource;

pub(crate) fn admit_if_pending<'intent>(
    context: &'intent HomeContext,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    Option<
        AdmittedHomeProgramTransitionCatalog<
            RelationalHomeProgramCatalogReadmissionSource<'intent>,
        >,
    >,
    ApplicationHomeTransitionIntentDenial,
> {
    use ApplicationHomeTransitionIntentDenial as Denial;
    request.consult().map_err(Denial::Execution)?;
    let Some(intent) = context
        .transition
        .committed_intent(&context.descriptor.binding)?
    else {
        return Ok(None);
    };
    let mut callers = worth_execution::ExecutionRequestArrayBuilder::allocate(
        *request,
        intent.assignments().len(),
    )
    .map_err(Denial::Backing)?;
    for (step, original) in intent.assignments().iter().enumerate() {
        request.consult().map_err(Denial::Execution)?;
        let commitment =
            worth_runtime_world::facade::home_transition_caller_association_commitment(
                request,
                intent.plan(),
                HomeProgramTransitionStepAssociation::from_recorded(
                    intent.plan().identity(),
                    step as u32,
                ),
                original,
            )
            .map_err(Denial::WorldAssociation)?;
        callers.push(commitment).map_err(Denial::Backing)?;
    }
    let source = RelationalHomeProgramCatalogReadmissionSource::from_recorded(
        intent.plan(),
        intent.native_predecessor(),
        intent.assignments(),
        callers.seal().map_err(Denial::Backing)?,
    );
    Ok(Some(
        context
            .transition
            .admit_catalog(intent.plan().commitment(), source),
    ))
}

pub(crate) fn admit<'intent>(
    context: &'intent HomeContext,
    request: &ExecutionRequest<'_, '_>,
) -> Result<
    AdmittedHomeProgramTransitionCatalog<RelationalHomeProgramCatalogReadmissionSource<'intent>>,
    ApplicationHomeTransitionIntentDenial,
> {
    admit_if_pending(context, request)?
        .ok_or(ApplicationHomeTransitionIntentDenial::IntentNotCommitted)
}
