//! Atomic field-plus-attachment authoring under the existing operation authority.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::model::WorthQueryApplicationEffectEntity;
use crate::domain_computation::primary_graph::ContentWriteDenial as Denial;
use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind;
use worth_foundational::facade::ContentRefId;
use worth_query_declaration::facade::application_schema::{
    ApplicationContentBinding, ApplicationContentFieldRef, ContentRefApplicationValueBinding,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationValueBindingCatalog as Catalog, WritableCapability,
};
use worth_relational::facade::content::RelationalContentAttachments;

impl<S, O, I, P> WorthQueryApplicationEffectProgramBuilder<S, O, I, P> {
    fn admit_content_target<E>(
        &self,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        locus: (&str, &str, &str),
        initialize: bool,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let (entity, aspect, field) = locus;
        self.validate_target(target, entity)?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: entity.to_owned(),
            aspect: aspect.to_owned(),
            field: field.to_owned(),
        })?;
        let locator = self.field_locator(entity, aspect, field)?;
        if initialize {
            let Some(position) = target.created_effect else {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                    field,
                ));
            };
            if !matches!(
                self.effects.get(position),
                Some(WorthQueryApplicationRealizedEffect::CreateEntity { .. })
            ) {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                    field,
                ));
            }
        } else {
            let EntityReference::Existing(entity_id) = target.reference else {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                    field,
                ));
            };
            self.require_parent_not_removed(entity, entity_id, &locator)?;
            self.require_field_observation(entity_id, &locator)?;
        }
        Ok(())
    }
    fn prepare_content<C: ApplicationContentBinding>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bindings: &Catalog,
        locus: (&str, &str, &str),
        input: &C::Value,
    ) -> Result<(RelationalContentAttachments, ContentRefId), Denial> {
        crate::domain_computation::primary_graph::content::ContentStaging {
            basis: self.read_set.lease.product().relational_basis(),
            execution,
            current: &self.content_attachments,
            bindings,
            denial: &mut self.content_denial,
        }
        .prepare(locus, input, |session, entity, aspect, field| {
            session.content_binding::<C>(entity, aspect, field)
        })
    }
    pub(in crate::domain_computation::primary_graph) fn initialize_content<E, A, F, W, Q, U, C>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bindings: &Catalog,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        content: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
        value: &C::Value,
    ) -> Result<(), Denial>
    where
        E: OperationCreates<O>,
        F: OperationWrites<O>
            + DeclaredApplicationFieldValue<
                Value = ContentRefId,
                Binding = ContentRefApplicationValueBinding,
            >,
        U: ApplicationFieldUnit,
        C: ApplicationContentBinding,
    {
        let field = content.field();
        self.admit_content_target(
            target,
            (field.entity(), field.aspect(), field.field()),
            true,
        )
        .map_err(Denial::Effect)?;
        let (staged, identity) = self.prepare_content::<C>(
            execution,
            bindings,
            (field.entity(), field.aspect(), field.field()),
            value,
        )?;
        self.initialize_field(target, field, identity)
            .map_err(Denial::Effect)?;
        self.content_attachments = staged;
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn write_content<E, A, F, W, Q, U, C>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bindings: &Catalog,
        target: &WorthQueryApplicationEffectEntity<S, E>,
        content: ApplicationContentFieldRef<S, E, A, F, W, Q, U, C>,
        value: &C::Value,
    ) -> Result<(), Denial>
    where
        F: OperationWrites<O>
            + DeclaredApplicationFieldValue<
                Value = ContentRefId,
                Binding = ContentRefApplicationValueBinding,
            >,
        W: WritableCapability,
        U: ApplicationFieldUnit,
        C: ApplicationContentBinding,
    {
        let field = content.field();
        self.admit_content_target(
            target,
            (field.entity(), field.aspect(), field.field()),
            false,
        )
        .map_err(Denial::Effect)?;
        let (staged, identity) = self.prepare_content::<C>(
            execution,
            bindings,
            (field.entity(), field.aspect(), field.field()),
            value,
        )?;
        self.write_field(target, field, identity)
            .map_err(Denial::Effect)?;
        self.content_attachments = staged;
        Ok(())
    }
}
