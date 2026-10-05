use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, WorthQueryApplicationOutputRole,
        WorthQueryApplicationOutputRoleFamily, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
        WorthQueryPreserveOutput,
    },
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::ApplicationEntityMarkerIdentity,
};
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationRelationRef, ApplicationSchema, DeclaredApplicationFieldValue, OperationReads,
    WritePosture,
};

use super::super::invariant_projection::WorthQueryPriorOutputRead;
use super::super::{
    HandlerExecutionDenial, WorthQueryCurrentOutputSelection, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantMutationTarget, WorthQueryInvariantProjectionTraversalDenial,
    WorthQueryInvariantRelation, WorthQueryPriorOutputFamilyMember, WorthQueryProducerOutputFamily,
};
use super::invariant::DecisionReader;

mod indexed_selection;
mod prior_member;

impl<Schema, Binding> DecisionReader<'_, '_, '_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    /// Resolve the entity `Family` currently outputs for `producer` through
    /// Query-owned lineage.
    #[allow(clippy::type_complexity)]
    pub fn current_output<Family, Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
    ) -> Result<WorthQueryCurrentOutputSelection<Schema, Family::Entity>, HandlerExecutionDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        Family::Source: ApplicationQueryBinding<Schema>,
        <Family::Source as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeBinding<Schema, Scope = Producer>,
        Producer:
            ApplicationEntityMarkerIdentity<Schema> + OperationReads<Binding::Operation> + 'static,
        Family::Entity: OperationReads<Binding::Operation>,
    {
        self.reader
            .current_output::<Family, Producer>(producer)
            .map_err(HandlerExecutionDenial::new)
    }

    /// Resolve the fixed role `Role` that `PriorBinding` last committed for
    /// this exact admitted scope.
    ///
    /// An exactly-one role yields its identity. An at-most-one role yields
    /// `Option`: `None` when the prior commit left the role unbound. A role
    /// the output contract of `PriorBinding` does not declare fails to
    /// compile.
    pub fn prior_output<PriorBinding, Role>(
        &mut self,
    ) -> Result<WorthQueryPriorOutputRead<Schema, Role>, HandlerExecutionDenial>
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = PriorBinding::Output>,
        Role::Entity: OperationReads<Binding::Operation>,
    {
        self.reader
            .prior_output::<PriorBinding, Role>()
            .map_err(HandlerExecutionDenial::new)
    }

    /// The outer `None` means `PriorBinding` has not published at the
    /// selected product coordinate; the inner read follows the role's
    /// cardinality, as for [`Self::prior_output`]. Missing context and stale
    /// entities still deny.
    pub fn prior_output_if_present<PriorBinding, Role>(
        &mut self,
    ) -> Result<Option<WorthQueryPriorOutputRead<Schema, Role>>, HandlerExecutionDenial>
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = PriorBinding::Output>,
        Role::Entity: OperationReads<Binding::Operation>,
    {
        self.reader
            .prior_output_if_present::<PriorBinding, Role>()
            .map_err(HandlerExecutionDenial::new)
    }

    /// Read the most recent preserved role, falling back to its initial create
    /// role only when no preserve correspondence exists for `PreserveBinding`.
    pub fn prior_output_preserved_or_created<PreserveBinding, Preserved, CreateBinding, Created>(
        &mut self,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Preserved::Entity>, HandlerExecutionDenial>
    where
        PreserveBinding: ApplicationMutationBinding<Schema>,
        Preserved: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = PreserveBinding::Output,
            Action = WorthQueryPreserveOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Preserved::Entity: OperationReads<Binding::Operation>,
        CreateBinding: ApplicationMutationBinding<Schema>,
        Created: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = CreateBinding::Output,
            Entity = Preserved::Entity,
            Action = WorthQueryCreateOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
    {
        select_prior_preserved_or_created(
            self.prior_output_if_present::<PreserveBinding, Preserved>(),
            || self.prior_output::<CreateBinding, Created>(),
        )
    }

    /// Resolve the complete live inventory of the family `Family` that
    /// `PriorBinding` last committed.
    #[allow(clippy::type_complexity)]
    pub fn prior_output_family<PriorBinding, Family>(
        &mut self,
    ) -> Result<
        Vec<WorthQueryPriorOutputFamilyMember<Schema, PriorBinding, Family::Entity>>,
        HandlerExecutionDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Binding::Operation>,
    {
        self.reader
            .prior_output_family::<PriorBinding, Family>()
            .map_err(HandlerExecutionDenial::new)
    }

    /// Resolve a complete prior family when `PriorBinding` has correspondence.
    #[allow(clippy::type_complexity)]
    pub fn prior_output_family_if_present<PriorBinding, Family>(
        &mut self,
    ) -> Result<
        Option<Vec<WorthQueryPriorOutputFamilyMember<Schema, PriorBinding, Family::Entity>>>,
        HandlerExecutionDenial,
    >
    where
        PriorBinding: ApplicationMutationBinding<Schema>,
        Family:
            WorthQueryApplicationOutputRoleFamily<Schema = Schema, Contract = PriorBinding::Output>,
        Family::Entity: OperationReads<Binding::Operation>,
    {
        self.reader
            .prior_output_family_if_present::<PriorBinding, Family>()
            .map_err(HandlerExecutionDenial::new)
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
        self.reader
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
        self.reader.decision_relations_from(relation, source)
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
        self.reader.decision_relations_to(relation, target)
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
        self.reader
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
