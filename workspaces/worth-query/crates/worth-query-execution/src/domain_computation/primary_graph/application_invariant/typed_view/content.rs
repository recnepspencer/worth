//! Content fields use the same installed format and admission as decision reads.
use super::*;
use crate::domain_computation::primary_graph::{
    content, AdmittedContent, ContentReadDenial as Denial, WorthQueryValueDecodeDenial as Stop,
};
use worth_query_declaration::facade::application_schema::{
    fragment::RecordSelection, ApplicationContentBinding, ApplicationContentFieldRef,
    ApplicationContentReferenceBinding,
};

impl<S> WorthQueryApplicationInvariantReadView<'_, S> {
    /// Reads only the selected field and its immutable frame, under this rule's
    /// declared entity closure, prepared scope and original work budget.
    pub fn content<E, F, C>(
        &self,
        entity: &WorthQueryApplicationInvariantEntity<S, E>,
        content: ApplicationContentFieldRef<F, C>,
    ) -> Result<Option<AdmittedContent<C>>, Denial>
    where
        F: RecordSelection<Entity = E>,
        F::Binding: ApplicationContentReferenceBinding,
        C: ApplicationContentBinding,
    {
        let selection = content::ContentFieldSelection::of::<F, C>(content.field());
        let result = self.content_field(&selection).and_then(|(_, field)| {
            self.check_entity(entity, field.kind)
                .map_err(|cause| Denial::InvariantAccess(Box::new(cause)))?;
            self.content_frame(entity.entity_id, &selection)
        });
        self.latch_content(&result);
        content::typed(result)
    }

    fn content_field(
        &self,
        selection: &content::ContentFieldSelection,
    ) -> Result<
        (
            &worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding,
            &super::super::content_fields::InvariantContentField,
        ),
        Denial,
    > {
        if let Some(cause) = self.content_decode.denial() {
            return Err(cause.into());
        }
        let column = selection.column;
        column
            .installed(self.content_fields.bindings.fragments())
            .map_err(|_| Denial::BindingContractMismatch)?;
        let binding = self
            .content_fields
            .bindings
            .field(column.entity(), column.aspect(), column.field())
            .filter(|binding| (selection.matches)(binding))
            .ok_or(Denial::BindingContractMismatch)?;
        let field = self
            .content_fields
            .fields
            .get(&column.locus())
            .ok_or(Denial::BindingContractMismatch)?;
        Ok((binding, field))
    }

    fn content_frame(
        &self,
        entity: EntityId,
        selection: &content::ContentFieldSelection,
    ) -> Result<Option<worth_relational::facade::content::RelationalContentFrame>, Denial> {
        let (binding, field) = self.content_field(selection)?;
        let locator = &field.locator;
        let column = selection.column;
        let state = self.aspects.entity_aspect_state(entity).map_err(|cause| {
            Denial::InvariantAccess(Box::new(map_structural_error(cause, "content field")))
        })?;
        self.reads.field();
        let raw = super::field_reads::field_material(
            state
                .get(locator.aspect().aspect_key())
                .map(|aspect| aspect.view()),
            field.aspect_absence,
            field.presence,
            locator.field_path().fields().first(),
        )
        .map_err(|_| Denial::RequiredFieldAbsent)?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        self.relations
            .consume_entity_field_content(entity, field.kind, locator, |execution, frame| {
                self.content_decode
                    .run(&execution, |session| {
                        let reference = (selection.decode)(
                            session,
                            column.entity(),
                            column.aspect(),
                            column.field(),
                            raw,
                        )
                        .ok_or_else(|| {
                            session
                                .denial()
                                .map(Denial::from)
                                .unwrap_or(Denial::BindingContractMismatch)
                        })?;
                        let mut frame = frame;
                        content::resolve_frame(&execution, session, binding, reference, &mut || {
                            frame.take().ok_or(Denial::UnavailableBasis)
                        })
                        .map(Some)
                    })
                    .map_err(Denial::from)?
            })
            .map_err(|cause| match cause {
                worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::Content(
                    cause,
                ) => Denial::Content(cause),
                cause => Denial::InvariantSource(Box::new(cause)),
            })?
    }

    fn latch_content<T>(&self, result: &Result<T, Denial>) {
        if let Err(cause) = result {
            self.content_decode.refuse(match cause {
                Denial::Content(cause) => Stop::Content(*cause),
                Denial::RequiredFieldAbsent => Stop::RequiredContentFieldAbsent,
                Denial::Admission(cause) => *cause,
                _ => Stop::BindingContractMismatch,
            });
        }
    }
}

impl<S> WorthQueryApplicationInvariantScopePlanner<'_, '_, S> {
    pub(in crate::domain_computation::primary_graph::application_invariant) fn check_content<
        Cause,
    >(
        &self,
    ) -> Result<(), super::super::WorthQueryApplicationInvariantError<Cause>> {
        self.content_decode.denial().map_or(Ok(()), |cause| {
            Err(super::super::WorthQueryApplicationInvariantError::Content(
                cause,
            ))
        })
    }
}
impl<S> WorthQueryApplicationInvariantContext<'_, '_, S> {
    pub(in crate::domain_computation::primary_graph::application_invariant) fn check_content<
        Cause,
    >(
        &self,
    ) -> Result<(), super::super::WorthQueryApplicationInvariantError<Cause>> {
        self.content_decode.denial().map_or(Ok(()), |cause| {
            Err(super::super::WorthQueryApplicationInvariantError::Content(
                cause,
            ))
        })
    }
}
