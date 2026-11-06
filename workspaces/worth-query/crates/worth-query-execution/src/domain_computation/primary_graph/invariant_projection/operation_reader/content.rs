//! Typed content reads retain the ordinary field dependency and sticky refusal.
use super::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
use crate::domain_computation::primary_graph::{
    AdmittedContent, ContentReadDenial as Denial, HandlerExecutionDenial,
    WorthQueryInvariantEntityIdentity, WorthQueryValueDecodeDenial as Stop,
};

use worth_query_declaration::facade::application_schema::{
    ApplicationContentBinding, ApplicationContentFieldRef,
};
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema, O> Reader<'_, '_, S, O> {
    pub(in crate::domain_computation::primary_graph) fn decision_content<E, F, C>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, E>,
        content: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, Denial>
    where
        F: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = E,
        >,

        F::Binding:
            worth_query_declaration::facade::application_schema::ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        let field = content.field();
        self.require_decision_field(identity, field)
            .map_err(|cause| Denial::Field(HandlerExecutionDenial::new(cause)))?;
        let session = self.reader.value_decode;
        let binding =
            session.content_binding::<C>(field.entity(), field.aspect(), field.field())?;
        let Some(locator) = self
            .reader
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
        else {
            session.refuse_content(Stop::BindingContractMismatch);
            return Err(Denial::BindingContractMismatch);
        };
        let Some(value) = self.reader.content_reference::<F::Binding>(
            identity.entity_id,
            identity.kind,
            (field.entity(), field.aspect(), field.field()),
            &locator,
        )?
        else {
            return Ok(None);
        };
        let Some(format) = binding.content_format() else {
            session.refuse_content(Stop::BindingContractMismatch);
            return Err(Denial::BindingContractMismatch);
        };
        let frame = match &self.reader.native {
            super::super::read_basis::NativeProjectionRead::Committed(_) => {
                crate::domain_computation::primary_graph::content::read_selected(
                    self.selected_product
                        .map(|product| product.relational_basis()),
                    self.reader.execution,
                    session,
                    value,
                    format,
                )?
            }
            super::super::read_basis::NativeProjectionRead::Applied { candidate, .. } => {
                let frame = candidate
                    .project_candidate()
                    .retain_entity_field_content(
                        self.reader.execution,
                        identity.entity_id,
                        identity.kind,
                        &locator,
                    )
                    .map_err(|cause| {
                        session.refuse_content(Stop::Content(cause));
                        Denial::Content(cause)
                    })?
                    .ok_or_else(|| {
                        session.refuse_content(Stop::ContentBasisUnavailable);
                        Denial::UnavailableBasis
                    })?;
                if frame.identity() != value {
                    session.refuse_content(Stop::BindingContractMismatch);
                    return Err(Denial::BindingContractMismatch);
                }
                frame
            }
        };
        AdmittedContent::new(self.reader.execution, frame, format)
            .map(Some)
            .map_err(|cause| {
                session.refuse_content(Stop::Content(cause));
                Denial::Content(cause)
            })
    }
}
