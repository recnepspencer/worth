//! The reader an owner reads its input through, lent from the handler that
//! runs the computation.

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationRelationRef, ApplicationSchema, DeclaredApplicationFieldValue, OperationReads,
    WritePosture,
};

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantProjectionTraversalDenial,
    WorthQueryInvariantRelation,
};

/// The handler's reader for the length of one owner call.
///
/// Every read is a decision read of the handler's operation, checked against
/// what the operation declares it reads. Query records the fact each read
/// depends on with the call that made it: the membership, one item's key or
/// one partition.
pub struct WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, Operation> {
    reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Operation,
    >,
}

impl<'call, 'reader, 'runtime, Schema, Operation>
    WorthQueryComputationReader<'call, 'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub(super) fn lend(
        reader: &'call mut WorthQueryApplicationOperationInvariantProjectionReader<
            'reader,
            'runtime,
            Schema,
            Operation,
        >,
    ) -> Self {
        Self { reader }
    }

    /// Reads a field's value.
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Option<Value>, WorthQueryComputationReadDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
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

/// Why a read through the lent reader was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryComputationReadDenial {
    Field(WorthQueryInvariantDecisionPlanDenial),
    Relations(WorthQueryInvariantProjectionTraversalDenial),
}

/// Why an owner call that reads the input has no answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryComputationInputDenial<Stopped> {
    /// The owner refused what it read.
    Owner(Stopped),
    /// A read was refused.
    Read(WorthQueryComputationReadDenial),
}

impl<Stopped> From<WorthQueryComputationReadDenial> for WorthQueryComputationInputDenial<Stopped> {
    fn from(denial: WorthQueryComputationReadDenial) -> Self {
        Self::Read(denial)
    }
}
