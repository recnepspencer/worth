//! Record handles open the same selected-product entity-resolution path.
use super::*;
use crate::domain_computation::primary_graph::fragment_view::FragmentDenial;
use worth_query_declaration::facade::application_schema::fragment::{
    Aspect, Entity, Fragment, In, RecordField,
};

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    pub fn resolve_record_entity<F, E, A, T, EP, AP>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        marker: F,
        field: RecordField<E, A, T>,
        value: T,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<S, E>, FragmentDenial>
    where
        F: Fragment,
        E: Entity + In<F::Members, EP, F>,
        A: Aspect + In<E::Aspects, AP, E>,
        T: 'static,
    {
        let locus = field.locus();
        let execution = phase
            .execution_request_for(&self.application().runtime.product_runtime)
            .map_err(|_| {
                FragmentDenial::Resolution(entity_denial(
                    WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::ForeignPhase),
                    locus.field(),
                ))
            })?;
        admit_request(request, locus.field()).map_err(FragmentDenial::Resolution)?;
        let fragment = self
            .application()
            .installed_schema()
            .value_bindings()
            .fragment(marker)
            .ok_or(FragmentDenial::NotInstalled { fragment: F::NAME })?;
        let aspect = fragment
            .aspect::<E, A>()
            .ok_or(FragmentDenial::MemberNotInstalled {
                entity: E::NAME,
                member: A::NAME,
            })?;
        let path = format!("{}.{}", aspect.path(), locus.field());
        if !aspect.fields().iter().any(|actual| {
            actual.recipe().locus() == locus
                && actual.recipe().value_type()
                    == crate::equality_type_id::EqualityTypeId::of::<T>().into_external()
                && actual.recipe().binding_type() == field.recipe().binding_type()
                && actual.equality()
        }) {
            return Err(FragmentDenial::InvalidField { path });
        }
        let value = field
            .encode(&value)
            .map_err(|cause| FragmentDenial::Encode {
                path,
                cause: cause.cause().clone(),
            })?;
        self.resolve_encoded_entity(
            &execution,
            locus.entity(),
            locus.aspect(),
            locus.field(),
            value,
            request,
            mode,
        )
        .map_err(FragmentDenial::Resolution)
    }
}
