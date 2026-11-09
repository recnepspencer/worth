//! Resolve actual installed operation contracts before admitting historical-read meaning.
use super::*;
use worth_query_installation::facade::WorthQueryOperationEffectContract;
pub(super) fn operation<S, O, I>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
where
    S: ApplicationSchema,
    O: ApplicationOperationMarkerIdentity<S> + 'static,
    O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    I: 'static,
{
    let operation = schema
        .installed_operation(ApplicationOperationRef::<S, O, I>::from_declaration())
        .map_err(|_| refused(O::IDENTIFIER))?;
    let contracts = operation.contracts();
    let read_only = contracts.touches().scopes().is_empty()
        && contracts.publication_touches().scopes().is_empty()
        && !contracts.emissions().is_declared()
        && matches!(
            contracts.effects(),
            WorthQueryOperationEffectContract::NotRequired
        )
        && !contracts.external_effect().is_declared()
        && contracts.body().is_none();
    if !read_only {
        return Err(refused(O::IDENTIFIER));
    }
    Ok(())
}
