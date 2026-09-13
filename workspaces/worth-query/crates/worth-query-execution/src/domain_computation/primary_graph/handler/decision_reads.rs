use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationRelationRef, ApplicationSchema, DeclaredApplicationFieldValue, EqualityPredicate,
    OperationReads, WritePosture,
};

use super::super::{
    HandlerExecutionDenial, WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantRelation,
};
use super::invariant::DecisionReader;

impl<Schema, Binding> DecisionReader<'_, '_, '_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Resolve a prior committed semantic output for this exact admitted scope.
    pub fn prior_output<PriorBinding, Entity, Action>(
        &mut self,
        role: super::super::WorthQueryApplicationOutputRole<PriorBinding, Entity, Action>,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        PriorBinding: 'static,
        Entity: 'static,
        Action: super::super::WorthQueryApplicationOutputAction,
    {
        self.reader()
            .prior_output(role)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Resolve through the admitted snapshot and retain the identity-field fact
    /// that later candidate authoring and stale-source comparison require.
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
        let identity = self
            .reader()
            .resolve_entity(field, value)
            .map_err(HandlerExecutionDenial::new)?;
        self.reader()
            .require_decision_field(&identity, field)
            .map_err(HandlerExecutionDenial::new)?;
        Ok(identity)
    }

    /// Read a value while retaining its exact field dependency for the attempt.
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Option<Value>, HandlerExecutionDenial>
    where
        Field: OperationReads<Binding::Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader()
            .decision_field(identity, field)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Read the complete admitted outgoing adjacency and retain absence as a
    /// source dependency for publication-time comparison.
    pub fn relations_from<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Binding::Operation>,
    {
        self.reader().decision_relations_from(relation, source)
    }

    /// Read the one target promised by an exactly-one outgoing cardinality
    /// declaration without allowing observed data to substitute for the contract.
    pub fn related_one<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<Schema, To>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Binding::Operation>,
    {
        let cardinality = relation.integrity().cardinality;
        if (cardinality.source_min, cardinality.source_max) != (Some(1), Some(1)) {
            return Err(
                WorthQueryInvariantProjectionTraversalDenial::cardinality_contract_mismatch(
                    relation.name(),
                ),
            );
        }
        let mut relations = self.relations_from(relation, source)?;
        match relations.len() {
            0 => Err(WorthQueryInvariantProjectionTraversalDenial::missing_target(relation.name())),
            1 => Ok(relations
                .pop()
                .expect("one relation remains after exact length check")
                .into_to()),
            _ => {
                Err(WorthQueryInvariantProjectionTraversalDenial::multiple_targets(relation.name()))
            }
        }
    }

    /// Admit an observed entity as a target for the candidate phase of this
    /// exact operation attempt.
    pub fn mutation_target<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryInvariantMutationTarget<Schema, Entity>, HandlerExecutionDenial> {
        self.reader()
            .mutation_target(identity)
            .map_err(HandlerExecutionDenial::new)
    }
}
