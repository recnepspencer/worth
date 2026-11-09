//! Entity resolution within the admitted read attempt.
use super::*;

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'_, '_, Schema, Operation, Input, Scope, Phase>
{
    pub fn resolve_entity<Aspect, Entity, Field, Value>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        field: Field,
        value: Value,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Entity>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let phase_request = phase
            .request_for_source(self.lease.handle().relational_runtime_instance_id())
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::ForeignPhase,
                    "read-attempt-phase",
                )
            })?;
        self.resolve_entity_with_request(&phase_request, field, value)
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_entity_with_request<
        Aspect,
        Entity,
        Field,
        Value,
    >(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        field: Field,
        value: Value,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Entity>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        self.check_value_decode()?;
        let execution = &self.execution;
        let column = self.layout.record_selection(field)?;
        Field::Binding::validate(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
                column.path(),
            )
        })?;
        let value = Field::Binding::encode(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
                field.field(),
            )
        })?;
        let resolved = self
            .lease
            .handle()
            .with_runtime(execution, |runtime| {
                self.entity_resolution
                    .at_snapshot(
                        runtime,
                        self.lease.snapshot(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .and_then(|truth| {
                        truth.resolve(
                            execution,
                            field.entity(),
                            field.aspect(),
                            field.field(),
                            value,
                        )
                    })
            })
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    field.field(),
                )
            })?;
        if !self.read_scope.admits(resolved.entity_id()) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutsideRealizedReadScope,
                field.entity(),
            ));
        }
        Ok(resolved.into_application_identity())
    }
}
