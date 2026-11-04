use super::*;
use worth_query_declaration::facade::application_schema::ApplicationEntityRef;

#[path = "read_access/lineage.rs"]
mod lineage;
#[path = "read_access/orbit.rs"]
mod orbit;
#[path = "read_access/walk.rs"]
mod walk;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub const fn version(&self) -> worth_relational::facade::identity::VersionId {
        self.reader.version()
    }

    /// The actual Native runtime that owns this projection. This is descriptive
    /// identity for comparing stored references, not admission or mutation authority.
    pub fn runtime_instance_id(&self) -> u64 {
        self.reader.native.runtime_instance_id()
    }

    /// Reacquire a descriptive historical endpoint at this operation's actual
    /// image. Live kind, phase, entity-read admission and authorization all apply.
    pub fn resolve_historical_entity<Entity>(
        &mut self,
        _entity: ApplicationEntityRef<Schema, Entity>,
        id: worth_foundational::facade::EntityId,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryCurrentOutputDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema> + OperationReads<Operation>,
    {
        let execution = *self.reader.execution;
        self.live_current_identity(
            &execution,
            Entity::IDENTIFIER,
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::new(id.partition_id.0),
                id.local_slot.0,
                id.generation.0,
            ),
        )
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
        let _execution = self.reader.execution;
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
        let _execution = self.reader.execution;
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
        let _execution = self.reader.execution;
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
            phase: self
                .reader
                .native
                .candidate_identity()
                .map(|id| id.qualification()),
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
        let _execution = self.reader.execution;
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
        let _execution = self.reader.execution;
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
        let _execution = self.reader.execution;
        self.reader
            .summarize_exclusive_incoming(relation, field, target)
    }
}
