//! Relation payload checks and original-request handoff have one concrete body.
use super::*;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_query_installation::facade::{
    AdmittedFieldBindingStop, ApplicationOperationProgramTarget as Target,
    WorthQueryInstalledApplicationValueBinding, WorthQueryInstalledApplicationValueBindingCatalog,
};
pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) struct CreatedRelation<
    'a,
> {
    pub name: &'a str,
    pub from: &'a str,
    pub to: &'a str,
    pub program: &'a Arc<()>,
    pub created_effect: usize,
}
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn initialize_relation_field(
        &mut self,
        original: ExecutionRequest<'_, '_>,
        bindings: &WorthQueryInstalledApplicationValueBindingCatalog,
        target: CreatedRelation<'_>,
        field: ColumnIdentity,
        endpoints: (&str, &str),
        matches_binding: fn(&WorthQueryInstalledApplicationValueBinding) -> bool,
        value: AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let column = field.installed(bindings.fragments()).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.field(),
            )
        })?;
        let failed = |kind| denial(kind, column.path());
        if !Arc::ptr_eq(self.program, target.program)
            || target.name != field.entity()
            || target.from != endpoints.0
            || target.to != endpoints.1
        {
            return Err(failed(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
            ));
        }
        let locator = self
            .layout
            .relation_field_locator(field.entity(), field.aspect(), field.field())
            .ok_or_else(|| failed(WorthQueryApplicationAttemptDenialKind::UndeclaredEffect))?;
        let kind = self
            .layout
            .relation(field.entity())
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
                        field.entity(),
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
                    .filter(|binding| matches_binding(binding))
                    .ok_or_else(|| {
                        failed(WorthQueryApplicationAttemptDenialKind::InvalidEffectValue)
                    })?;
                self.admit_program_target(&Target::InitializeRelationField {
                    relation: field.entity().to_owned(),
                    from: endpoints.0.to_owned(),
                    to: endpoints.1.to_owned(),
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
            .relation_field_locator(field.entity(), field.aspect(), field.field())
            .expect("validated installed relation field")
            .clone();
        let Some(WorthQueryApplicationRealizedEffect::CreateRelation { fields, .. }) =
            self.effects.get_mut(target.created_effect)
        else {
            unreachable!("sealed create index remains installed")
        };
        fields.insert(locator, value);
        Ok(())
    }
}
