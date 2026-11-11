use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCreatedRelation;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_installation::facade::OperationLinks;

impl<Schema, Binding> CandidateWriter<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Creates the same ordinary link and returns its program-owned initialization handle.
    pub fn link_for_initialization<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        key: impl Into<String>,
        from: &WorthQueryApplicationEffectEntity<Schema, From>,
        to: &WorthQueryApplicationEffectEntity<Schema, To>,
    ) -> Result<
        WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Relation: OperationLinks<Binding::Operation>,
    {
        self.execution.consult().map_err(|cause| {
            WorthQueryApplicationAttemptDenial::relational_read_stopped(
                cause.into(),
                relation.name(),
            )
        })?;
        self.candidate
            .link_for_initialization(self.execution, relation, key, from, to)
    }

    /// Moves an explicitly prepared typed value into this new relation. The
    /// original handler request supplies control stops; user encoding took
    /// place when constructing `ApplicationEncodedScalarValue`.
    pub fn initialize_relation_field<Relation, From, To, Field>(
        &mut self,
        target: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        field: Field,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Binding::Operation>,
        Field: worth_query_declaration::facade::application_schema::fragment::RelationSelection<
            Entity = Relation,
            From = From,
            To = To,
        >,
    {
        self.candidate.initialize_relation_field(
            self.execution,
            self.value_bindings,
            target,
            field,
            value,
        )
    }
}
