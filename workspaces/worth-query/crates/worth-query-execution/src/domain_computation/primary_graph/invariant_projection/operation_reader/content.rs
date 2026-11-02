//! Typed content reads retain the ordinary field dependency and sticky refusal.
use super::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
use crate::domain_computation::primary_graph::{
    AdmittedContent, ContentReadDenial as Denial, HandlerExecutionDenial,
    WorthQueryInvariantEntityIdentity, WorthQueryValueDecodeDenial as Stop,
};
use worth_foundational::facade::ContentRefId;
use worth_query_declaration::facade::application_schema::{
    ApplicationContentBinding, ApplicationContentFieldRef, ContentRefApplicationValueBinding,
};
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationSchema, DeclaredApplicationFieldValue, OperationReads,
    WritePosture,
};

impl<S: ApplicationSchema, O> Reader<'_, '_, S, O> {
    pub(in crate::domain_computation::primary_graph) fn decision_content<E, A, F, W, Q, U, C>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, E>,
        content: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
    ) -> Result<Option<AdmittedContent<C>>, Denial>
    where
        F: OperationReads<O>
            + DeclaredApplicationFieldValue<
                Value = ContentRefId,
                Binding = ContentRefApplicationValueBinding,
            >,
        W: WritePosture,
        U: ApplicationFieldUnit,
        C: ApplicationContentBinding,
    {
        let field = content.field();
        self.require_decision_field(identity, field)
            .map_err(|cause| Denial::Field(HandlerExecutionDenial::new(cause)))?;
        let session = self.reader.value_decode;
        let binding =
            session.content_binding::<C>(field.entity(), field.aspect(), field.field())?;
        let locator = self
            .reader
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .expect("the decision field was admitted above")
            .clone();
        let Some(value) = self.reader.content_reference(
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
        let frame = crate::domain_computation::primary_graph::content::read_selected(
            self.selected_product
                .map(|product| product.relational_basis()),
            self.reader.execution,
            session,
            value,
            format,
        )?;
        AdmittedContent::new(self.reader.execution, frame, format)
            .map(Some)
            .map_err(|cause| {
                session.refuse_content(Stop::Content(cause));
                Denial::Content(cause)
            })
    }
}
