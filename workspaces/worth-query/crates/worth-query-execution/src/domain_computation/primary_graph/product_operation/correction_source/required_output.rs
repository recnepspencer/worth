//! Typed required-root derivation keeps delivery selection inside the native owner.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationRequiredOutputRoot,
};
type Connection<S, R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn derive_required_output_reapplication<C, Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        alternative: &WorthQuerySelectedCorrectionAlternative<'_, Schema>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<
        (
            ApplicationEncodedInput<
                <C::Source as ApplicationMutationBinding<Schema>>::InputBinding,
            >,
            std::sync::Arc<WorthQueryPreparedCorrectionSource>,
        ),
        WorthQueryCorrectionDerivationDenial,
    >
    where
        C: ApplicationCorrectionBinding<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        C::Source: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
        <C::Source as ApplicationMutationBinding<Schema>>::Input:
            worth_query_declaration::facade::application_operation::ApplicationMutationInputScope<
                Schema,
                C::Source,
            >,
        <C::Source as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <C::Source as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        super::reapplication::derive::<Schema, C>(
            self,
            phase,
            principal,
            request,
            alternative,
            key,
            maximum_history,
            maximum_results,
            Some(crate::equality_type_id::EqualityTypeId::of::<Root>()),
        )
    }
    /// Derive one checked required-output inverse after fresh historical disclosure.
    /// The installed target must own this root and the actual inverse source binding.
    pub fn derive_required_output_correction<C, Root>(
        &self,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> Result<(
        ApplicationEncodedInput<<ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::InputBinding>,
        std::sync::Arc<WorthQueryPreparedCorrectionSource>,
    ), WorthQueryCorrectionDerivationDenial>
    where
        C: ApplicationCorrectionBinding<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        ApplicationInverseBinding<Schema, C>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
        <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<Schema, <ApplicationInverseBinding<Schema, C> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    {
        derivation::derive_correction_inverse::<Schema, C>(
            self,
            access,
            phase,
            principal,
            request,
            source,
            key,
            maximum_history,
            maximum_results,
            Some(crate::equality_type_id::EqualityTypeId::of::<Root>()),
        )
    }
}
