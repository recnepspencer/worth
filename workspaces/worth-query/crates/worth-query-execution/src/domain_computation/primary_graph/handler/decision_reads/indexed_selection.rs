use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, DeclaredApplicationFieldValue, EqualityPredicate, OperationReads,
    WritePosture,
};

use super::DecisionReader;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryInvariantEntityIdentity,
};

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    /// Read the complete equality result under a finite candidate budget.
    /// Presence and absence are retained for publication-time comparison.
    /// An overflowing result denies; it never masquerades as a complete set.
    pub fn select_entities<Entity, Aspect, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader
            .decision_select_entities(field, value, candidate_limit)
    }

    /// Resolve one identity while retaining both its field and complete
    /// equality selection, so a concurrent competing match makes it stale.
    pub fn resolve_entity<Entity, Aspect, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.resolve_optional_entity(field, value)?.ok_or_else(|| {
            HandlerExecutionDenial::new(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::UnknownEntity,
                field.field(),
            ))
        })
    }

    /// Resolve zero or one identity while retaining absence as a predicate
    /// dependency. Ambiguity and resource exhaustion remain denials.
    pub fn resolve_optional_entity<Entity, Aspect, Field, Value, Write, Unit>(
        &mut self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let mut matches = self
            .select_entities(field, value, 2)
            .map_err(|denial| resolution_denial(denial, field.field()))?;
        if matches.len() > 1 {
            return Err(HandlerExecutionDenial::new(
                WorthQueryEntityResolutionDenial::new(
                    WorthQueryEntityResolutionDenialKind::AmbiguousEntity,
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

fn resolution_denial(denial: HandlerExecutionDenial, subject: &str) -> HandlerExecutionDenial {
    match denial.downcast::<WorthQueryEntityResolutionDenial>() {
        Ok(denial)
            if matches!(
                denial.kind(),
                WorthQueryEntityResolutionDenialKind::CandidateLimitExceeded { .. }
            ) =>
        {
            HandlerExecutionDenial::new(WorthQueryEntityResolutionDenial::new(
                WorthQueryEntityResolutionDenialKind::AmbiguousEntity,
                subject,
            ))
        }
        Ok(denial) => HandlerExecutionDenial::new(denial),
        Err(denial) => denial,
    }
}
