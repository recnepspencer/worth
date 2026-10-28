use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCreatedRelation;
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEncodedScalarValue, ApplicationFieldMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationRelationFieldRef, OperationLinks, OperationWrites,
};

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
    pub fn initialize_relation_field<Relation, From, To, Aspect, Field, Write, Unit>(
        &mut self,
        target: &WorthQueryApplicationCreatedRelation<Schema, Relation, From, To>,
        field: ApplicationRelationFieldRef<Schema, Relation, From, To, Aspect, Field, Write, Unit>,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Relation: OperationLinks<Binding::Operation>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Relation>,
        Field: ApplicationFieldMarkerIdentity<Schema, Relation, Aspect>
            + OperationWrites<Binding::Operation>,
        Unit: ApplicationFieldUnit,
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
