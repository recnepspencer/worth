//! Every failed prelude read remains with the loan even if the owner catches it.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryInvariantRelation;
use worth_query_declaration::facade::application_schema::{
    fragment::RecordSelection, ApplicationReadableScalarValueBinding, ApplicationRelationRef,
};

impl<S, B, C> WorthQueryProducerInputReader<'_, '_, '_, S, B, C>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    C: WorthQueryProducerInputCaptureDefinition<S>,
{
    pub fn field<Entity, Aspect, Field, Value>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<S, Entity>,
        field: Field,
    ) -> Result<Option<Value>, HandlerExecutionDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let result = self.reader.decision_field(identity, field);
        self.checked(result)
    }

    pub fn relations_from<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<S, Relation, From, To>,
        source: &WorthQueryInvariantEntityIdentity<S, From>,
    ) -> Result<Vec<WorthQueryInvariantRelation<S, Relation, From, To>>, HandlerExecutionDenial>
    where
        Relation: OperationReads<B::Operation>,
    {
        let result = self.reader.decision_relations_from(relation, source);
        self.checked(result)
    }

    pub fn relations_to<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<S, Relation, From, To>,
        target: &WorthQueryInvariantEntityIdentity<S, To>,
    ) -> Result<Vec<WorthQueryInvariantRelation<S, Relation, From, To>>, HandlerExecutionDenial>
    where
        Relation: OperationReads<B::Operation>,
    {
        let result = self.reader.decision_relations_to(relation, target);
        self.checked(result)
    }

    /// Complete equality selection retains nonmatches and absence; it never truncates.
    pub fn select_entities<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
        value: Value,
        candidate_limit: usize,
    ) -> Result<Vec<WorthQueryInvariantEntityIdentity<S, Entity>>, HandlerExecutionDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
    {
        let result = self
            .reader
            .decision_select_entities(field, value, candidate_limit);
        self.checked(result)
    }

    /// Presence retains the required selected field as well as the whole predicate.
    pub fn resolve_optional_entity<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
        value: Value,
    ) -> Result<Option<WorthQueryInvariantEntityIdentity<S, Entity>>, HandlerExecutionDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let result = self.reader.resolve_optional_decision_entity(field, value);
        self.checked(result)
    }

    pub fn resolve_entity<Entity, Aspect, Field, Value>(
        &mut self,
        field: Field,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<S, Entity>, HandlerExecutionDenial>
    where
        Field: RecordSelection<Entity = Entity, Aspect = Aspect, Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let result = self.resolve_optional_entity(field, value)?.ok_or_else(|| {
            use crate::domain_computation::primary_graph::{
                WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind as Kind,
                WorthQueryEntityResolutionRefusalKind as Refusal,
            };
            WorthQueryEntityResolutionDenial::new(
                Kind::Refused(Refusal::UnknownEntity),
                field.field(),
            )
        });
        self.checked(result)
    }
}
