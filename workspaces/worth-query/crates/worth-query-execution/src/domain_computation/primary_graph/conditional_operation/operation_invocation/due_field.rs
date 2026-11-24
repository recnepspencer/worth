use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationSchema, ApplicationSchemaMember,
    WorthQueryTemporalIntentDueValue,
};

use super::{WorthQueryTemporalInvocationFailure, WorthQueryTemporalInvocationFailureKind};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};

/// Marker indicating that no recurring due-coordinate field is configured on
/// [`super::WorthQueryTemporalOperationExecution`]. Returning `Reschedule`
/// fails closed before any effect commits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryNoTemporalDueField;

/// Typed due-coordinate field selection installed on
/// [`super::WorthQueryTemporalOperationExecution::with_due_field`] so Query
/// atomically observes and writes the next due coordinate alongside `revision`
/// and `lifecycle`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalDueField<Field>(pub(super) Field);

impl<Field> WorthQueryTemporalDueField<Field> {
    pub const fn new(field: Field) -> Self {
        Self(field)
    }
}

/// Authoritative due-field validation, read observation, and atomic effect-writing contract.
pub trait WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>:
    Copy + Send + Sync + 'static
where
    Schema: ApplicationSchema,
{
    fn validate_installed(&self, members: &[ApplicationSchemaMember]) -> Result<(), String>;

    fn validate_reschedule(
        &self,
        next_due_nanoseconds: u64,
    ) -> Result<(), WorthQueryTemporalInvocationFailure>;

    fn observe_due<Operation>(
        &self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        intent: &WorthQueryInvariantEntityIdentity<Schema, IntentEntity>,
    ) -> Result<(), String>;

    fn write_due<Operation, Input, Scope>(
        &self,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
        target: &WorthQueryApplicationEffectEntity<Schema, IntentEntity>,
        next_due_nanoseconds: u64,
    ) -> Result<(), super::super::WorthQueryConditionalReentryFailure>;
}

impl<Schema, IntentEntity> WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>
    for WorthQueryNoTemporalDueField
where
    Schema: ApplicationSchema,
{
    fn validate_installed(&self, _members: &[ApplicationSchemaMember]) -> Result<(), String> {
        Ok(())
    }

    fn validate_reschedule(
        &self,
        _next_due_nanoseconds: u64,
    ) -> Result<(), WorthQueryTemporalInvocationFailure> {
        Err(WorthQueryTemporalInvocationFailure::new(
            WorthQueryTemporalInvocationFailureKind::InvocationRejected,
            "recurring temporal reschedule requires a configured due field on WorthQueryTemporalOperationExecution",
        ))
    }

    fn observe_due<Operation>(
        &self,
        _reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        _intent: &WorthQueryInvariantEntityIdentity<Schema, IntentEntity>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn write_due<Operation, Input, Scope>(
        &self,
        _effects: &mut WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
        _target: &WorthQueryApplicationEffectEntity<Schema, IntentEntity>,
        _next_due_nanoseconds: u64,
    ) -> Result<(), super::super::WorthQueryConditionalReentryFailure> {
        Err(
            super::super::WorthQueryConditionalReentryFailure::Invocation(
                WorthQueryTemporalInvocationFailure::new(
                    WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                    "recurring temporal reschedule requires a configured due field",
                ),
            ),
        )
    }
}

impl<Schema, IntentEntity, Field> WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>
    for WorthQueryTemporalDueField<Field>
where
    Schema: ApplicationSchema,
    Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = IntentEntity,
        > + Copy
        + Send
        + Sync
        + 'static,
    Field::Binding: ApplicationReadableScalarValueBinding<Value = Field::Value>
        + WorthQueryTemporalIntentDueValue,
    Field::Value: Clone + Send + 'static,
{
    fn validate_installed(&self, members: &[ApplicationSchemaMember]) -> Result<(), String> {
        let entity = self.0.entity();
        let aspect = self.0.aspect();
        let field = self.0.field();
        let scalar_family = self.0.scalar_family();
        let found = members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Field {
                    entity: m_entity,
                    aspect: m_aspect,
                    field: m_field,
                    scalar_family: m_family,
                    ..
                } if m_entity == entity
                    && m_aspect == aspect
                    && m_field == field
                    && *m_family == scalar_family
            )
        });
        if found {
            Ok(())
        } else {
            Err(format!(
                "temporal operation due field `{field}` is not installed"
            ))
        }
    }

    fn validate_reschedule(
        &self,
        next_due_nanoseconds: u64,
    ) -> Result<(), WorthQueryTemporalInvocationFailure> {
        if Field::Binding::from_due_nanoseconds(next_due_nanoseconds).is_some() {
            Ok(())
        } else {
            Err(WorthQueryTemporalInvocationFailure::new(
                WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                "rescheduled due coordinate is not representable by the configured due field",
            ))
        }
    }

    fn observe_due<Operation>(
        &self,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        intent: &WorthQueryInvariantEntityIdentity<Schema, IntentEntity>,
    ) -> Result<(), String> {
        reader
            .decision_field(intent, self.0)
            .map_err(|denial| denial.to_string())?
            .ok_or_else(|| "temporal intent due coordinate is absent".to_string())?;
        Ok(())
    }

    fn write_due<Operation, Input, Scope>(
        &self,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
        target: &WorthQueryApplicationEffectEntity<Schema, IntentEntity>,
        next_due_nanoseconds: u64,
    ) -> Result<(), super::super::WorthQueryConditionalReentryFailure> {
        let due_value = Field::Binding::from_due_nanoseconds(next_due_nanoseconds).ok_or_else(
            || {
                super::super::WorthQueryConditionalReentryFailure::Invocation(
                    WorthQueryTemporalInvocationFailure::new(
                        WorthQueryTemporalInvocationFailureKind::InvocationRejected,
                        "rescheduled due coordinate is not representable by the configured due field",
                    ),
                )
            },
        )?;
        effects
            .write_field(target, self.0, due_value)
            .map_err(|denial| {
                super::super::WorthQueryConditionalReentryFailure::ApplicationAttempt(denial.kind())
            })
    }
}
