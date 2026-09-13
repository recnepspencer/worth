use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputRole,
    WorthQueryPriorOutputDenial, WorthQueryPriorOutputDenialKind,
};
use worth_query_installation::facade::ApplicationEntityRef;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn prior_output<Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryPriorOutputDenial>
    where
        Binding: 'static,
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation> + 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        let scope = self.operation_scope.as_ref().ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                role.name(),
            )
        })?;
        let occurrence = self.reader.selected_product_occurrence.ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                role.name(),
            )
        })?;
        let generation = self.reader.selected_product_generation.ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                role.name(),
            )
        })?;
        let binding_type = std::any::TypeId::of::<Binding>();
        let selection_work = usize::from(
            !self
                .reader
                .prior_output_bindings
                .contains_key(&binding_type),
        );
        let required_work = selection_work + 1;
        if !self.reader.work_budget.can_afford(required_work) {
            return Err(WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::WorkBudgetExceeded,
                role.name(),
            ));
        }
        if selection_work == 1 {
            let correspondence = self
                .reader
                .output_lineage
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .resolve_binding::<Binding>(scope, occurrence, generation);
            self.reader.work_budget.consume(1);
            self.reader.work.record_output_lineage_selection();
            let correspondence = correspondence.ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::Unavailable,
                    role.name(),
                )
            })?;
            self.reader
                .prior_output_bindings
                .insert(binding_type, correspondence);
        }
        self.reader.work_budget.consume(1);
        self.reader.work.record_output_lineage_role_lookup();
        let resolution = self
            .reader
            .prior_output_bindings
            .get(&binding_type)
            .expect("prior output binding was selected")
            .entity(role)
            .map_err(|denial| WorthQueryPriorOutputDenial::projection(role.name(), denial))?;
        let record = self
            .reader
            .runtime
            .read_truth()
            .visible_entity_at_version(resolution.entity_id(), self.reader.snapshot.version_id())
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::OutputUnavailable,
                    role.name(),
                )
            })?;
        let entity = self
            .reader
            .layout
            .entity_name(record.kind.kind_id)
            .ok_or_else(|| {
                WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::EntityMismatch,
                    role.name(),
                )
            })?;
        if entity != Entity::IDENTIFIER {
            return Err(WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::EntityMismatch,
                role.name(),
            ));
        }
        self.reader.realized_scope.record(resolution.entity_id());
        let identity = WorthQueryInvariantEntityIdentity {
            entity_id: resolution.entity_id(),
            kind: record.kind.kind_id,
            entity: Arc::from(entity),
            authority_identity: self.reader.authority_identity,
            _marker: PhantomData,
        };
        self.require_decision_entity(
            &identity,
            ApplicationEntityRef::from_schema_identifier(Entity::IDENTIFIER),
        )
        .map_err(|_| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                role.name(),
            )
        })?;
        Ok(identity)
    }

    pub const fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.reader.version()
    }

    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
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
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader.resolve_entity(field, value)
    }

    pub fn resolve_optional_entity<Aspect, Entity, Field, Value, Write, Unit>(
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
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader.resolve_optional_entity(field, value)
    }

    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Option<Value>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader.field(identity, field)
    }

    pub fn mutation_target<Entity>(
        &self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantMutationTarget<Schema, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let admission_identity = self.admission_identity.ok_or_else(|| {
            WorthQueryInvariantProjectionTraversalDenial::mutation_target_unavailable(
                identity.entity_name(),
            )
        })?;
        if identity.authority_identity != self.reader.authority_identity {
            return Err(
                WorthQueryInvariantProjectionTraversalDenial::foreign_identity(
                    identity.entity_name(),
                ),
            );
        }
        Ok(WorthQueryInvariantMutationTarget {
            entity_id: identity.entity_id,
            entity: Arc::clone(&identity.entity),
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity.clone(),
            admission_identity,
            _marker: PhantomData,
        })
    }

    pub fn relations_from<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        self.reader.relations_from(relation, from)
    }

    pub fn relations_to<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        to: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        self.reader.relations_to(relation, to)
    }

    pub fn summarize_exclusive_incoming<
        Relation,
        From,
        To,
        Aspect,
        Field,
        Value,
        Write,
        Equality,
        Unit,
    >(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        field: ApplicationFieldRef<Schema, From, Aspect, Field, Value, Write, Equality, Unit>,
        target: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<super::super::WorthQueryInvariantAggregate<Value>, WorthQueryInvariantAggregateDenial>
    where
        Relation: OperationReads<Operation>,
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationSignedAggregateValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        self.reader
            .summarize_exclusive_incoming(relation, field, target)
    }
}
