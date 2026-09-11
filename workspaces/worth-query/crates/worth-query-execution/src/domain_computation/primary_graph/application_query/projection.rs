use std::marker::PhantomData;

use worth_query_declaration::facade::application_query::{
    ApplicationQueryMarkerIdentity, ApplicationQueryOptionalResultFieldRef,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationCardinality,
    ApplicationQueryResultRelationRef, ApplicationQueryResultTraversal, ExactlyOneResult,
    ManyResults, OptionalOneResult,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableType;
use worth_query_installation::facade::{
    ApplicationFieldUnit, ApplicationReadableScalarValueBinding, OptionalApplicationFieldValue,
    RequiredApplicationFieldValue, WritePosture,
};

mod disclosed;
mod projected_tree;

pub use disclosed::{WorthQueryApplicationDisclosed, WorthQueryApplicationOmission};

pub(super) use projected_tree::{
    WorthQueryApplicationDisclosedProjectionNode, WorthQueryApplicationDisclosedProjectionTree,
    WorthQueryApplicationProjectedField, WorthQueryApplicationProjectedRelation,
    WorthQueryApplicationProjectionNode, WorthQueryApplicationWorkingProjectionTree,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationProjectionDenialKind {
    FieldNotProjected,
    FieldContractMismatch,
    FieldTypeMismatch,
    FieldOmitted,
    RelationNotProjected,
    RelationContractMismatch,
    RelationCardinalityMismatch,
    RelationOmitted,
    DomainProjectionRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationProjectionDenial {
    kind: WorthQueryApplicationProjectionDenialKind,
    subject: String,
}

/// Query-owned authoritative row presented to domain projection code.
///
/// Construction is private. Reads require query-bound result selectors whose
/// slot, path meaning, field or relation contract, and cardinality match the
/// installed application query.
pub struct WorthQueryApplicationProjectionRow<'row, Schema, Query> {
    node: WorthQueryApplicationDisclosedProjectionNode<'row>,
    governance: &'row super::disclosure::WorthQueryApplicationQueryGovernance,
    _marker: PhantomData<fn() -> (Schema, Query)>,
}

pub struct WorthQueryApplicationProjectionRows<'row, Schema, Query> {
    rows: &'row [WorthQueryApplicationProjectionNode],
    disclosure_parent: WorthQueryApplicationDisclosedProjectionNode<'row>,
    governance: &'row super::disclosure::WorthQueryApplicationQueryGovernance,
    _marker: PhantomData<fn() -> (Schema, Query)>,
}

impl WorthQueryApplicationProjectionDenial {
    pub fn reject(subject: impl Into<String>) -> Self {
        Self::new(
            WorthQueryApplicationProjectionDenialKind::DomainProjectionRejected,
            subject,
        )
    }

    pub const fn kind(&self) -> WorthQueryApplicationProjectionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    fn new(kind: WorthQueryApplicationProjectionDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl<'row, Schema, Query> WorthQueryApplicationProjectionRow<'row, Schema, Query> {
    pub fn field<Slot, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        selector: ApplicationQueryResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Result<Value, WorthQueryApplicationProjectionDenial>
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_field(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::FieldOmitted)
    }

    pub fn optional_field<Slot, Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &self,
        selector: ApplicationQueryOptionalResultFieldRef<
            Query,
            Slot,
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Result<Option<Value>, WorthQueryApplicationProjectionDenial>
    where
        Field: OptionalApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_optional_field(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::FieldOmitted)
    }

    pub fn optional<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            OptionalOneResult,
        >,
    ) -> Result<
        Option<WorthQueryApplicationProjectionRow<'_, Schema, Query>>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let relation = self.relation(&selector)?;
        match relation.rows() {
            [] => Ok(None),
            [row] => Ok(Some(WorthQueryApplicationProjectionRow::new(
                self.node.child(row),
                self.governance,
            ))),
            _ => Err(relation_cardinality_denial(relation)),
        }
    }

    pub fn one<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ExactlyOneResult,
        >,
    ) -> Result<
        WorthQueryApplicationProjectionRow<'_, Schema, Query>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let relation = self.relation(&selector)?;
        match relation.rows() {
            [row] => Ok(WorthQueryApplicationProjectionRow::new(
                self.node.child(row),
                self.governance,
            )),
            _ => Err(relation_cardinality_denial(relation)),
        }
    }

    pub fn many<Slot, Relation, From, To, Direction>(
        &self,
        selector: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ManyResults,
        >,
    ) -> Result<
        WorthQueryApplicationProjectionRows<'_, Schema, Query>,
        WorthQueryApplicationProjectionDenial,
    >
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        let relation = self.relation(&selector)?;
        Ok(WorthQueryApplicationProjectionRows {
            rows: relation.rows(),
            disclosure_parent: self.node,
            governance: self.governance,
            _marker: PhantomData,
        })
    }

    fn relation<Slot, Relation, From, To, Direction, Cardinality>(
        &self,
        selector: &ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        >,
    ) -> Result<&WorthQueryApplicationProjectedRelation, WorthQueryApplicationProjectionDenial>
    where
        Direction: ApplicationQueryResultTraversal,
        Cardinality: ApplicationQueryResultRelationCardinality,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.disclosed_relation(selector)?
            .into_required(WorthQueryApplicationProjectionDenialKind::RelationOmitted)
    }

    pub(super) const fn new(
        node: WorthQueryApplicationDisclosedProjectionNode<'row>,
        governance: &'row super::disclosure::WorthQueryApplicationQueryGovernance,
    ) -> Self {
        Self {
            node,
            governance,
            _marker: PhantomData,
        }
    }
}

impl<'row, Schema, Query> WorthQueryApplicationProjectionRows<'row, Schema, Query> {
    pub const fn len(&self) -> usize {
        self.rows.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn iter(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationProjectionRow<'_, Schema, Query>> {
        self.rows.iter().map(|row| {
            WorthQueryApplicationProjectionRow::new(
                self.disclosure_parent.child(row),
                self.governance,
            )
        })
    }
}

pub trait WorthQueryApplicationProjection<Schema, Query>: Sized {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial>;
}

fn projection_denial(
    kind: WorthQueryApplicationProjectionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationProjectionDenial {
    WorthQueryApplicationProjectionDenial::new(kind, subject)
}

fn relation_cardinality_denial(
    relation: &WorthQueryApplicationProjectedRelation,
) -> WorthQueryApplicationProjectionDenial {
    projection_denial(
        WorthQueryApplicationProjectionDenialKind::RelationCardinalityMismatch,
        relation.result_path(),
    )
}

impl std::fmt::Display for WorthQueryApplicationProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application-query projection denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationProjectionDenial {}
