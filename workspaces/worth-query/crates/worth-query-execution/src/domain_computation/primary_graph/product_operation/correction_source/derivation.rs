//! Application inverse derivation follows the exact source's fresh disclosure.
use super::disclosure::{
    disclose_correction_source, WorthQueryCorrectionDerivationDenial as Denial,
};
use super::*;
use crate::domain_computation::application_aftermath::retained_preimage::WorthQueryRetainedPreImage;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_aftermath::{
        ApplicationCorrectionBinding, ApplicationCorrectionPreImage, ApplicationInverseBinding,
        DeclaredPreImageLocus,
    },
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
    let interpretation =
        super::compatibility::require_compatible_source::<Schema, C>(target, phase, source)?;
    let retained = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .retained_governed_input_carrier()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let input = retained
        .encoded_input::<<C::Source as ApplicationMutationBinding<Schema>>::InputBinding>()
        .ok_or(Denial::UnsupportedSourceMeaning)?;
    let preimage = source
        .occurrence()
        .commit_evidence()
        .retained_preimage()
        .ok_or(Denial::MissingPreImage)?;
    let inverse =
        C::derive_inverse(input.input(), &DisclosedPreImage(preimage)).map_err(Denial::Domain)?;
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
    )?;
    Ok((input, std::sync::Arc::new(prepared)))
}

struct DisclosedPreImage<'a>(&'a WorthQueryRetainedPreImage);
impl<Schema> ApplicationCorrectionPreImage<Schema> for DisclosedPreImage<'_> {
    fn field(
        &self,
        locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        self.0
            .fields()
            .iter()
            .find(|field| {
                field.locus().entity() == locus.entity()
                    && field.locus().aspect() == locus.aspect()
                    && field.locus().field() == locus.field()
            })
            .map(|field| field.value())
    }
}
