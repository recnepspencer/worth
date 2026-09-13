use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputRole,
    WorthQueryPriorOutputDenial, WorthQueryPriorOutputDenialKind,
};

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
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        let scope = self.operation_scope.as_ref().ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                role.name(),
            )
        })?;
        let selected_commit = self.reader.selected_commit.ok_or_else(|| {
            WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                role.name(),
            )
        })?;
        if self.reader.output_lineage_ancestry.is_none() {
            let ancestry = self
                .reader
                .runtime
                .history()
                .ancestor_closure_by_commit_id_order(selected_commit);
            if !self.reader.work_budget.can_afford(ancestry.len()) {
                return Err(WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::WorkBudgetExceeded,
                    role.name(),
                ));
            }
            self.reader.work_budget.consume(ancestry.len());
            self.reader
                .work
                .record_output_lineage_ancestry(ancestry.len());
            self.reader.output_lineage_ancestry = Some(ancestry);
        }
        let ancestry = self
            .reader
            .output_lineage_ancestry
            .as_ref()
            .expect("output lineage ancestry was initialized");
        if !self.reader.work_budget.can_afford(ancestry.len()) {
            return Err(WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::WorkBudgetExceeded,
                role.name(),
            ));
        }
        let resolution = self
            .reader
            .output_lineage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolve(ancestry, scope, role)?;
        self.reader.work_budget.consume(resolution.commit_probes);
        self.reader
            .work
            .record_output_lineage_lookup(resolution.commit_probes);
        let record = self
            .reader
            .runtime
            .read_truth()
            .visible_entity_at_version(resolution.entity, self.reader.snapshot.version_id())
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
        self.reader.realized_scope.record(resolution.entity);
        Ok(WorthQueryInvariantEntityIdentity {
            entity_id: resolution.entity,
            kind: record.kind.kind_id,
            entity: Arc::from(entity),
            authority_identity: self.reader.authority_identity,
            _marker: PhantomData,
        })
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
