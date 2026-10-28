use super::*;
use std::sync::Arc;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEncodedScalarValue, ApplicationFieldMarkerIdentity,
};
use worth_query_installation::facade::{
    AdmittedFieldBindingStop, ApplicationFieldUnit, ApplicationOperationProgramTarget as Target,
    ApplicationRelationFieldRef, ApplicationRelationRef, OperationLinks, OperationWrites,
    WorthQueryInstalledApplicationValueBindingCatalog,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn link_for_initialization<
        Relation,
        From,
        To,
    >(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<
        WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Relation: OperationLinks<Operation>,
    {
        let position = self.link_at_target_index(
            &Target::Link {
                relation: relation.name().to_owned(),
                from: relation.from().to_owned(),
                to: relation.to().to_owned(),
            },
            key,
            from,
            to,
        )?;
        original.consult().map_err(|cause| {
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial::relational_read_stopped(cause.into(),relation.name())
        })?;
        Ok(WorthQueryApplicationCreatedRelation {
            relation,
            created_effect: position,
            program: Arc::clone(&self.program),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn initialize_relation_field<
        Relation,
        From,
        To,
        Aspect,
        Field,
        Write,
        Unit,
    >(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
        target: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        field: ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Operation>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
        Field:
            ApplicationFieldMarkerIdentity<Schema, Relation, Aspect> + OperationWrites<Operation>,
        Unit: ApplicationFieldUnit,
    {
        let failed = |kind| denial(kind, field.field());
        if !Arc::ptr_eq(&self.program, &target.program)
            || target.relation.name() != field.relation().name()
            || target.relation.from() != field.relation().from()
            || target.relation.to() != field.relation().to()
        {
            return Err(failed(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
            ));
        }
        let locator = self
            .layout
            .relation_field_locator(field.relation().name(), field.aspect(), field.field())
            .ok_or_else(|| failed(WorthQueryApplicationAttemptDenialKind::UndeclaredEffect))?;
        let kind = self
            .layout
            .relation(field.relation().name())
            .ok_or_else(|| failed(WorthQueryApplicationAttemptDenialKind::UndeclaredEffect))?
            .kind;
        let Some(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: actual,
            fields,
            ..
        }) = self.effects.get(target.created_effect)
        else {
            return Err(failed(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
            ));
        };
        if *actual != kind {
            return Err(failed(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
            ));
        }
        if fields.contains_key(locator) {
            return Err(failed(
                WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
            ));
        }
        let scope_denied = |cause: worth_execution::WorkCeilingDenial| {
            WorthQueryApplicationAttemptDenial::relational_read_stopped(cause.into(), field.field())
        };
        let mut initialized = None;
        let outcome = original.run_native_with_checkpoint(|context| {
            initialized = Some((|| {
                context
                    .poll()
                    .map_err(|cause| scope_denied(WorkCeilingDenial::Stopped(cause)))?;
                bindings
                    .relation_field_admitted(
                        field.relation().name(),
                        field.aspect(),
                        field.field(),
                        &mut |_, _| context.poll(),
                    )
                    .map_err(|cause| match cause {
                        AdmittedFieldBindingStop::Admission(cause) => {
                            scope_denied(WorkCeilingDenial::Stopped(cause))
                        }
                        AdmittedFieldBindingStop::AccountingOverflow => failed(
                            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
                        ),
                    })?
                    .filter(|binding| binding.matches_binding::<Field::Binding>())
                    .ok_or_else(|| {
                        failed(WorthQueryApplicationAttemptDenialKind::InvalidEffectValue)
                    })?;
                self.admit_program_target(&Target::InitializeRelationField {
                    relation: field.relation().name().to_owned(),
                    from: field.relation().from().to_owned(),
                    to: field.relation().to().to_owned(),
                    aspect: field.aspect().to_owned(),
                    field: field.field().to_owned(),
                })?;
                context
                    .poll()
                    .map_err(|cause| scope_denied(WorkCeilingDenial::Stopped(cause)))
            })());
        });
        match (outcome, initialized) {
            (Ok(_), Some(result)) => result,
            (
                Err(WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::NestedStopped)),
                Some(Err(cause)),
            ) => Err(cause),
            (Err(cause), _) => Err(scope_denied(cause)),
            (Ok(_), None) => unreachable!("completed initializer callback"),
        }?;
        // Finish the original request handoff before installing the field value.
        let locator = self
            .layout
            .relation_field_locator(field.relation().name(), field.aspect(), field.field())
            .expect("validated installed relation field")
            .clone();
        let Some(WorthQueryApplicationRealizedEffect::CreateRelation { fields, .. }) =
            self.effects.get_mut(target.created_effect)
        else {
            unreachable!("sealed create index remains installed")
        };
        fields.insert(locator, value.into_foundational_value());
        Ok(())
    }
}
