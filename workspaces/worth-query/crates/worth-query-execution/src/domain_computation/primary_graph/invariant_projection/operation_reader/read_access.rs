use super::*;

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
        self.reader.runtime.runtime_instance_id()
    }

    pub fn resolve_entity<Aspect, Entity, Field, Value>(
        &mut self,

        field: Field,
        value: Value,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let _execution = self.reader.execution;
        self.reader.resolve_entity(field, value)
    }

    pub fn resolve_optional_entity<Aspect, Entity, Field, Value>(
        &mut self,

        field: Field,
        value: Value,
    ) -> Result<
        Option<WorthQueryInvariantEntityIdentity<Schema, Entity>>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
    {
        let _execution = self.reader.execution;
        self.reader.resolve_optional_entity(field, value)
    }

    #[inline(always)]
    pub fn field<Entity, Aspect, Field, Value>(
        &mut self,

        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Option<Value>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
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
        Ok(super::mutation_target::target(
            identity.native_identity(),
            self.reader.authority_identity,
            self.runtime_authority,
            self.binding_identity,
            self.admission_identity,
        )?
        .into_typed())
    }

    #[inline(always)]
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

    #[inline(always)]
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

    pub fn summarize_exclusive_incoming<Relation, From, To, Aspect, Field, Value>(
        &mut self,

        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        field: Field,
        target: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<super::super::WorthQueryInvariantAggregate<Value>, WorthQueryInvariantAggregateDenial>
    where
        Relation: OperationReads<Operation>,
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = From,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationSignedAggregateValueBinding,
    {
        let _execution = self.reader.execution;
        self.reader
            .summarize_exclusive_incoming(relation, field, target)
    }
}
