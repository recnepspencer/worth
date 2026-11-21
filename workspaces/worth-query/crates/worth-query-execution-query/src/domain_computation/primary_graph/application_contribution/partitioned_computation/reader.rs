//! The reader an owner reads its input through, lent from the handler that
//! runs the computation.

mod content;
mod derived_relation;
mod orbit;
mod output_availability;
mod structural_walk;

use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationRelationRef, ApplicationSchema,
    OperationReads,
};

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantRelation,
};

/// The handler's reader for the length of one owner call.
///
/// Every read is a decision read of the handler's operation, checked against
/// what the operation declares it reads. Query records the fact each read
/// depends on with the call that made it: the membership, one item's key or
/// one partition.
///
/// The reader belongs to the request, so a kernel, which is `Fn + Sync`,
/// cannot capture it, even when its schema and operation are `Sync`:
///
/// ```compile_fail
/// use worth_query_execution::facade::application_contribution::WorthQueryComputationReader;
///
/// fn kernel_captures<Schema: Sync, Operation: Sync>(
///     reader: &WorthQueryComputationReader<'_, '_, '_, Schema, Operation>,
/// ) {
///     fn kernel(_: impl Fn() + Sync) {}
///     kernel(move || {
///         let _held = reader;
///     });
/// }
/// ```
pub struct WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, Operation> {
    reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Operation,
    >,
    source_query: Option<
        &'call crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
            'call,
            Schema,
        >,
    >,
}

impl<'call, 'reader, 'runtime, Schema, Operation>
    WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn lend(
        reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
            'reader,
            'runtime,
            Schema,
            Operation,
        >,
        source_query: Option<
            &'call crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
                'call,
                Schema,
            >,
        >,
    ) -> Self {
        Self {
            reader,
            source_query,
        }
    }

    /// Reads a field's value.
    pub fn field<Entity, Aspect, Field, Value>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Result<Option<Value>, WorthQueryComputationReadDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        self.reader
            .decision_field(identity, field)
            .map_err(WorthQueryComputationReadDenial::Field)
    }

    /// Reads the complete outgoing adjacency of `source`.
    pub fn relations_from<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryComputationReadDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        self.reader
            .decision_relations_from(relation, source)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }

    /// Reads the complete incoming adjacency of `target`.
    pub fn relations_to<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        target: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryComputationReadDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        self.reader
            .decision_relations_to(relation, target)
            .map_err(WorthQueryComputationReadDenial::Relations)
    }
}

pub use worth_query_execution_lineage::facade::primary_graph::application_contribution::partitioned_computation::denial::WorthQueryComputationReadDenial;
