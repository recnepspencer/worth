//! Exclusive incoming aggregate projection.
//!
//! The public reader method is deliberately only the phase coordinator. Each
//! phase owns a sealed output whose fields cannot be assembled by a sibling.

mod cache_resolution;
mod completed_scan;
mod execution;
mod validated_plan;

#[cfg(test)]
mod tests;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationRelationRef, ApplicationSchema,
    ApplicationSignedAggregateValueBinding, DeclaredApplicationFieldValue, WritePosture,
};

use super::{WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantAggregateDenialKind {
    RelationNotInstalled,
    FieldNotInstalled,
    ForeignIdentity,
    WorkBudgetExceeded,
    InvalidScalar,
    ArithmeticOverflow,
    SourceCountOverflow,
    AmbiguousSourceRelation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantAggregateDenial {
    kind: WorthQueryInvariantAggregateDenialKind,
    member: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantAggregate<Value> {
    value: Value,
    source_count: u64,
}

impl<Value> WorthQueryInvariantAggregate<Value> {
    pub const fn value(&self) -> &Value {
        &self.value
    }

    pub const fn source_count(&self) -> u64 {
        self.source_count
    }

    pub fn into_value(self) -> Value {
        self.value
    }
}

impl WorthQueryInvariantAggregateDenial {
    pub const fn kind(&self) -> WorthQueryInvariantAggregateDenialKind {
        self.kind
    }

    pub fn member(&self) -> &str {
        &self.member
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionReader<'_, Schema>
where
    Schema: ApplicationSchema,
{
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
    ) -> Result<WorthQueryInvariantAggregate<Value>, WorthQueryInvariantAggregateDenial>
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationSignedAggregateValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let plan = validated_plan::ValidatedAggregatePlan::validate(self, relation, field, target)?;
        execution::execute::<Schema, Field::Binding>(self, plan)
    }
}

fn denial(
    kind: WorthQueryInvariantAggregateDenialKind,
    member: impl Into<String>,
) -> WorthQueryInvariantAggregateDenial {
    WorthQueryInvariantAggregateDenial {
        kind,
        member: member.into(),
    }
}
