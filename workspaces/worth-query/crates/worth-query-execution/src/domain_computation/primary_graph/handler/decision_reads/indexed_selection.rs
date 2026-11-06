use crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{ApplicationReadableScalarValueBinding, ApplicationSchema};

use super::DecisionReader;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryInvariantEntityIdentity,
};
use worth_query_declaration::facade::application_schema::OperationReads;

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Reacquire one historical endpoint through ordinary current entity admission.
    pub fn resolve_historical_entity<Entity>(
        &mut self,
        entity: worth_query_installation::facade::ApplicationEntityRef<Schema, Entity>,
        id: worth_foundational::facade::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        Entity:
            worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity<
                    Schema,
                > + OperationReads<Binding::Operation>,
    {
        self.reader
            .resolve_historical_entity(entity, id)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Read the complete equality result under a finite candidate budget.
    /// Presence and absence are retained for publication-time comparison.
    /// An overflowing result denies; it never masquerades as a complete set.
    pub fn select_entities<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let _execution = self.execution().execution_request().map_err(|cause| {
            HandlerExecutionDenial::new(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                    cause,
                ),
            )
        })?;
        self.reader
            .decision_select_entities(field, value, candidate_limit)
    }

    /// Resolve one identity while retaining both its field and complete
    /// equality selection, so a concurrent competing match makes it stale.
    pub fn resolve_entity<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        self.resolve_optional_entity(field, value)?.ok_or_else(|| {
            HandlerExecutionDenial::new(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::UnknownEntity),
                field.field(),
            ))
        })
    }

    /// Resolve zero or one identity while retaining absence as a predicate
    /// dependency. Ambiguity and resource exhaustion remain denials.
    pub fn resolve_optional_entity<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
        value: Value,
    ) -> Result<Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let mut matches = self
            .select_entities(field, value, 2)
            .map_err(|denial| resolution_denial(denial, field.field()))?;
        if matches.len() > 1 {
            return Err(HandlerExecutionDenial::new(
                WorthQueryEntityResolutionDenial::new(
                    WorthQueryEntityResolutionDenialKind::Refused(
                        ResolutionRefusal::AmbiguousEntity,
                    ),
                    field.field(),
                ),
            ));
        }
        let identity = matches.pop();
        if let Some(ref identity) = identity {
            self.reader
                .require_decision_field(identity, field)
                .map_err(HandlerExecutionDenial::new)?;
        }
        Ok(identity)
    }
}

pub(in crate::domain_computation::primary_graph) fn resolution_denial(
    denial: HandlerExecutionDenial,
    subject: &str,
) -> HandlerExecutionDenial {
    match denial.downcast::<WorthQueryEntityResolutionDenial>() {
        Ok(denial)
            if matches!(
                denial.kind(),
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::CandidateLimitExceeded { .. }
                )
            ) =>
        {
            HandlerExecutionDenial::new(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::AmbiguousEntity),
                subject,
            ))
        }
        Ok(denial) => HandlerExecutionDenial::new(denial),
        Err(denial) => denial,
    }
}
