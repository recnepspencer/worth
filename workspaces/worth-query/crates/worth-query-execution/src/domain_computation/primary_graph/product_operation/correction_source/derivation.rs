//! Application inverse derivation follows the exact source's fresh disclosure.
use super::disclosure::{
    disclose_correction_source, WorthQueryCorrectionDerivationDenial as Denial,
};
use super::*;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_aftermath::{ApplicationCorrectionBinding, ApplicationInverseBinding},
    application_operation::{
        ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationIntent,
        ApplicationMutationScopeResolution,
    },
};

pub(in crate::domain_computation::primary_graph) fn derive_correction_inverse<Schema, C>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    key: &WorthQueryCorrectionCommandKey,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<
    (
        ApplicationEncodedInput<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>,
        std::sync::Arc<super::WorthQueryPreparedCorrectionSource>,
    ),
    Denial,
>
where
    Schema: ApplicationSchema,
    C: ApplicationCorrectionBinding<Schema>,
    <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
{
    let ancestry = disclose_correction_source::<Schema, C>(
        target,
        phase,
        principal,
        request,
        source,
        maximum_history,
        maximum_results,
    )?;
    let lineage = super::lineage::reverse_lineage(target, phase, source)?;
    let checked = super::compatibility::require_compatible_source::<Schema, C>(
        target,
        phase,
        source,
        ancestry,
        required_root,
    )?;
    if let Some(material) = source.occurrence().home_material()
        && target
            .application()
            .runtime
            .primary_provider
            .recover_correction_command(material.capture.original, key.identity())
            .map_err(Denial::Material)?
            .is_some()
    {
        return super::WorthQueryPreparedCorrectionSource::recover::<
            Schema,
            C,
            ApplicationInverseBinding<Schema, C>,
        >(
            checked,
            phase,
            principal,
            request,
            lineage,
            key,
            C::recover_inverse_input,
        );
    }
    let workflow = super::preparation::prepare_workflow::<Schema, C>(target, phase, source)?;
    let (bytes, identity) = source
        .occurrence()
        .input()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let input =
        super::material::recorded_input::recover_bytes(bytes, identity, C::recover_source_input)?;
    let execution = phase
        .execution_request_for(&target.application().runtime.product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    let inverse = super::material::derive::<Schema, C>(source, execution, input.input())?;
    let input = target.application().capture_operation_input_in_advancement::<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>(
        phase, access, request, inverse.input(),
    ).map_err(Denial::InputCapture)?;
    let prepared = super::WorthQueryPreparedCorrectionSource::retain::<
        Schema,
        C,
        ApplicationInverseBinding<Schema, C>,
    >(
        checked,
        phase,
        principal,
        request,
        None,
        lineage,
        key,
        &input,
        inverse.scope_binding(),
        workflow,
    )?;
    Ok((input, std::sync::Arc::new(prepared)))
}
