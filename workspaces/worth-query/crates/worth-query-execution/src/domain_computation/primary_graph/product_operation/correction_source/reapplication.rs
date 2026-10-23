//! Reapplication adopts the exact captured source input under fresh source admission.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryCorrectionLineageBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;

pub(super) fn selected_lineage<S>(
    alternative: &WorthQuerySelectedCorrectionAlternative<'_, S>,
) -> Result<WorthQueryCorrectionLineageBinding, WorthQueryCorrectionDerivationDenial> {
    Ok(WorthQueryCorrectionLineageBinding::reapply(
        alternative.root.clone(),
        alternative.selected.clone(),
        alternative
            .active
            .as_ref()
            .ok_or(WorthQueryCorrectionDerivationDenial::NoActiveReversal)?
            .clone(),
    ))
}

pub(super) fn derive<S, C>(
    target: &WorthQuerySelectedProductOperation<'_, S>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    alternative: &WorthQuerySelectedCorrectionAlternative<'_, S>,
    key: &WorthQueryCorrectionCommandKey,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
    required_root: Option<crate::equality_type_id::EqualityTypeId>,
) -> Result<
    (
        ApplicationEncodedInput<<C::Source as ApplicationMutationBinding<S>>::InputBinding>,
        std::sync::Arc<WorthQueryPreparedCorrectionSource>,
    ),
    WorthQueryCorrectionDerivationDenial,
>
where
    S: ApplicationSchema,
    C: ApplicationCorrectionBinding<S>,
    <C::Source as ApplicationMutationBinding<S>>::Input:
        ApplicationMutationInputScope<S, C::Source>,
    <C::Source as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    use WorthQueryCorrectionDerivationDenial as Denial;
    let source = &alternative.source;
    let ancestry = disclosure::disclose_correction_source::<S, C>(
        target,
        phase,
        principal,
        request,
        source,
        maximum_history,
        maximum_results,
    )?;
    let selected =
        target.select_alternative_ancestry(phase, &alternative.selected, maximum_history)?;
    let active = alternative
        .active
        .as_ref()
        .ok_or(Denial::NoActiveReversal)?;
    let active_ancestry = target.select_alternative_ancestry(phase, active, maximum_history)?;
    let execution = phase
        .execution_request_for(&target.application().product_runtime)
        .map_err(|_| Denial::ForeignPhase)?;
    if target
        .application()
        .primary_provider
        .consumption_of_reversal_at_basis(&execution, target.product().relational_basis(), active)
        .map_err(Denial::History)?
        .is_some()
    {
        return Err(Denial::ActiveReversalConsumed);
    }
    let interpretation =
        super::compatibility::require_reapplication_program::<S, C>(target, source, required_root)?;
    let workflow = super::preparation::prepare_workflow::<S, C>(target, phase, source)?;
    let retained = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .retained_governed_input_carrier()
        .ok_or(Denial::SemanticMaterialUnavailable)?;
    let input = retained
        .encoded_input::<<C::Source as ApplicationMutationBinding<S>>::InputBinding>()
        .ok_or(Denial::UnsupportedSourceMeaning)?
        .clone();
    let scope =
        <<C::Source as ApplicationMutationBinding<S>>::Input as ApplicationMutationInputScope<
            S,
            C::Source,
        >>::scope_for_binding(input.input());
    let prepared = WorthQueryPreparedCorrectionSource::retain::<S, C, C::Source>(
        target,
        phase,
        principal,
        request,
        source,
        ancestry,
        Some((selected, active_ancestry)),
        selected_lineage(alternative)?,
        key,
        &input,
        scope,
        interpretation,
        required_root,
        workflow,
    )?;
    Ok((input, std::sync::Arc::new(prepared)))
}

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    pub fn select_reapplication_input<C: ApplicationCorrectionBinding<S>>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        alternative: &WorthQuerySelectedCorrectionAlternative<'_, S>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        WorthQueryCorrectionActionInputSelection<S, C::Source>,
        WorthQueryCorrectionDerivationDenial,
    > {
        super::replay::select_reapplication_input::<S, C>(
            self,
            phase,
            principal,
            request,
            alternative,
            key,
            maximum_history,
            maximum_results,
        )
    }

    pub fn derive_correction_reapplication<C>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        alternative: &WorthQuerySelectedCorrectionAlternative<'_, S>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        (
            ApplicationEncodedInput<<C::Source as ApplicationMutationBinding<S>>::InputBinding>,
            std::sync::Arc<WorthQueryPreparedCorrectionSource>,
        ),
        WorthQueryCorrectionDerivationDenial,
    >
    where
        C: ApplicationCorrectionBinding<S>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        derive::<S, C>(
            self,
            phase,
            principal,
            request,
            alternative,
            key,
            maximum_history,
            maximum_results,
            None,
        )
    }
}
