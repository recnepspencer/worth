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
    let interpretation = super::compatibility::require_compatible_source::<Schema, C>(
        target,
        phase,
        source,
        required_root,
    )?;
    let retained = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .retained_governed_input_carrier()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let input = retained
        .encoded_input::<<C::Source as ApplicationMutationBinding<Schema>>::InputBinding>()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let execution = phase
        .execution_request_for(&target.application().product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    let inverse = super::material::derive::<Schema, C>(source, execution, input.input())?;
    let input = target.application().capture_operation_input_in_advancement::<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>(
        phase, access, request, inverse.input(),
    ).map_err(Denial::InputCapture)?;
    let prepared = super::WorthQueryPreparedCorrectionSource::retain::<Schema, C>(
        target,
        phase,
        principal,
        request,
        source,
        ancestry,
        key,
        &input,
        inverse.scope_binding(),
        interpretation,
        required_root,
    )?;
    Ok((input, std::sync::Arc::new(prepared)))
}
