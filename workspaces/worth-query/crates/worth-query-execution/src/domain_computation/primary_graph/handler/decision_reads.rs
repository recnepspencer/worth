use worth_query_declaration::facade::{
    application_operation::ApplicationMutationBinding,
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::ApplicationEntityMarkerIdentity,
};
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
    /// Resolve one producer's current generated output through Query-owned lineage.
    pub fn current_output<Family, Producer, Entity>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
        role: super::super::WorthQueryCurrentOutputRole<Family, Entity>,
    ) -> Result<
        super::super::WorthQueryCurrentOutputSelection<Schema, Entity>,
        HandlerExecutionDenial,
    >
    where
        Family: super::super::WorthQueryProducerOutputFamily<Schema>,
        Family::Source: ApplicationQueryBinding<Schema>,
        <Family::Source as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer>,
        Producer:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
        Entity:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
    {
        self.reader()
            .current_output(producer, role)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Resolve a prior committed semantic output for this exact admitted scope.
    pub fn prior_output<PriorBinding, Entity, Action>(
        &mut self,
        role: super::super::WorthQueryApplicationOutputRole<PriorBinding, Entity, Action>,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        PriorBinding: 'static,
        Entity:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
        Action: super::super::WorthQueryApplicationOutputAction,
    {
        self.reader()
            .prior_output(role)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Absence means this binding has not published at the selected product
    /// coordinate. Missing context, stale entities and invalid roles still deny.
    pub fn prior_output_if_present<PriorBinding, Entity, Action>(
        &mut self,
        role: super::super::WorthQueryApplicationOutputRole<PriorBinding, Entity, Action>,
    ) -> Result<Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>, HandlerExecutionDenial>
    where
        PriorBinding: 'static,
        Entity:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
        Action: super::super::WorthQueryApplicationOutputAction,
    {
        self.reader()
            .prior_output_if_present(role)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Read the most recent preserved role, falling back to its initial create
    /// role only when no preserve correspondence exists for this binding.
    pub fn prior_output_preserved_or_created<PreserveBinding, CreateBinding, Entity>(
        &mut self,
        preserved: super::super::WorthQueryApplicationOutputRole<
            PreserveBinding,
            Entity,
            super::super::WorthQueryPreserveOutput,
        >,
        created: super::super::WorthQueryApplicationOutputRole<
            CreateBinding,
            Entity,
            super::super::WorthQueryCreateOutput,
        >,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, HandlerExecutionDenial>
    where
        PreserveBinding: 'static,
        CreateBinding: 'static,
        Entity:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
    {
        select_prior_preserved_or_created(self.prior_output_if_present(preserved), || {
            self.prior_output(created)
        })
    }

    /// Resolve the complete live inventory of one declared prior output family.
    pub fn prior_output_family<PriorBinding, Entity>(
        &mut self,
        family: super::super::WorthQueryApplicationOutputRoleFamily<PriorBinding, Entity>,
    ) -> Result<
        Vec<super::super::WorthQueryPriorOutputFamilyMember<Schema, PriorBinding, Entity>>,
        HandlerExecutionDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Entity:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
    {
        self.reader()
            .prior_output_family(family)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Resolve a complete prior family when this exact binding has correspondence.
    pub fn prior_output_family_if_present<PriorBinding, Entity>(
        &mut self,
        family: super::super::WorthQueryApplicationOutputRoleFamily<PriorBinding, Entity>,
    ) -> Result<
        Option<Vec<super::super::WorthQueryPriorOutputFamilyMember<Schema, PriorBinding, Entity>>>,
        HandlerExecutionDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Entity:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
    {
        self.reader()
            .prior_output_family_if_present(family)
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

    /// Resolve an admitted identity if present, retaining its identity-field
    /// dependency when found. Only an unknown identity becomes absence.
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
        let identity = self
            .reader()
            .resolve_optional_entity(field, value)
            .map_err(HandlerExecutionDenial::new)?;
        if let Some(ref identity) = identity {
            self.reader()
                .require_decision_field(identity, field)
                .map_err(HandlerExecutionDenial::new)?;
        }
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

    /// Read the complete admitted incoming adjacency and retain absence as a
    /// source dependency for publication-time comparison.
    pub fn relations_to<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        target: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Binding::Operation>,
    {
        self.reader().decision_relations_to(relation, target)
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

    /// Read the one source promised by an exactly-one incoming cardinality
    /// declaration without allowing observed data to substitute for the contract.
    pub fn related_one_incoming<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        target: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        WorthQueryInvariantEntityIdentity<Schema, From>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Binding::Operation>,
    {
        let cardinality = relation.integrity().cardinality;
        if (cardinality.target_min, cardinality.target_max) != (Some(1), Some(1)) {
            return Err(
                WorthQueryInvariantProjectionTraversalDenial::cardinality_contract_mismatch(
                    relation.name(),
                ),
            );
        }
        let mut relations = self.relations_to(relation, target)?;
        match relations.len() {
            0 => Err(WorthQueryInvariantProjectionTraversalDenial::missing_source(relation.name())),
            1 => Ok(relations
                .pop()
                .expect("one relation remains after exact length check")
                .into_from()),
            _ => {
                Err(WorthQueryInvariantProjectionTraversalDenial::multiple_sources(relation.name()))
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

fn select_prior_preserved_or_created<Value, Error>(
    preserved: Result<Option<Value>, Error>,
    created: impl FnOnce() -> Result<Value, Error>,
) -> Result<Value, Error> {
    match preserved? {
        Some(value) => Ok(value),
        None => created(),
    }
}

#[cfg(test)]
mod prior_output_selection_tests;
