use std::marker::PhantomData;

use crate::application_schema::ApplicationRelationRef;

use super::result_slot_key::ApplicationQueryResultRelationSlotContract;
use super::{
    ApplicationQueryCardinality, ApplicationQueryMarkerIdentity, ApplicationQueryResultSlotKey,
    ApplicationQueryResultTraversal, ApplicationQueryResultTraversalDirection,
    ForwardResultTraversal, ReverseResultTraversal,
};
use crate::portable_identity::WorthQueryPortableType;

mod cardinality_seal {
    pub trait Sealed {}
}

pub trait ApplicationQueryResultRelationCardinality: cardinality_seal::Sealed {
    const CARDINALITY: ApplicationQueryCardinality;
}

pub struct OptionalOneResult;
pub struct ExactlyOneResult;
pub struct ManyResults;

impl cardinality_seal::Sealed for OptionalOneResult {}
impl cardinality_seal::Sealed for ExactlyOneResult {}
impl cardinality_seal::Sealed for ManyResults {}

impl ApplicationQueryResultRelationCardinality for OptionalOneResult {
    const CARDINALITY: ApplicationQueryCardinality = ApplicationQueryCardinality::OptionalOne;
}

impl ApplicationQueryResultRelationCardinality for ExactlyOneResult {
    const CARDINALITY: ApplicationQueryCardinality = ApplicationQueryCardinality::ExactlyOne;
}

impl ApplicationQueryResultRelationCardinality for ManyResults {
    const CARDINALITY: ApplicationQueryCardinality = ApplicationQueryCardinality::Many;
}

pub struct ApplicationQueryResultRelationRef<
    Query,
    Slot,
    Schema,
    Relation,
    From,
    To,
    Direction,
    Cardinality,
> {
    output_name: &'static str,
    relation: &'static str,
    from: &'static str,
    to: &'static str,
    _marker: PhantomData<
        fn() -> (
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            Cardinality,
        ),
    >,
}

impl<Query, Slot, Schema, Relation, From, To, Direction, Cardinality> Clone
    for ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        Direction,
        Cardinality,
    >
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Query, Slot, Schema, Relation, From, To, Direction, Cardinality> Copy
    for ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        Direction,
        Cardinality,
    >
{
}

impl<Query, Slot, Schema, Relation, From, To, Direction, Cardinality>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        Direction,
        Cardinality,
    >
where
    Direction: ApplicationQueryResultTraversal,
    Cardinality: ApplicationQueryResultRelationCardinality,
{
    fn new(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self {
            output_name,
            relation: relation.name(),
            from: relation.from(),
            to: relation.to(),
            _marker: PhantomData,
        }
    }

    pub const fn output_name(&self) -> &'static str {
        self.output_name
    }

    pub const fn relation(&self) -> &'static str {
        self.relation
    }

    pub const fn from(&self) -> &'static str {
        self.from
    }

    pub const fn to(&self) -> &'static str {
        self.to
    }

    pub const fn direction(&self) -> ApplicationQueryResultTraversalDirection {
        Direction::DIRECTION
    }

    pub const fn parent(&self) -> &'static str {
        match Direction::DIRECTION {
            ApplicationQueryResultTraversalDirection::Forward => self.from,
            ApplicationQueryResultTraversalDirection::Reverse => self.to,
        }
    }

    pub const fn child(&self) -> &'static str {
        match Direction::DIRECTION {
            ApplicationQueryResultTraversalDirection::Forward => self.to,
            ApplicationQueryResultTraversalDirection::Reverse => self.from,
        }
    }

    pub fn query_type(&self) -> &'static str
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
    {
        Query::QUERY_TYPE_NAME
    }

    pub fn slot_type(&self) -> &'static str
    where
        Slot: WorthQueryPortableType,
    {
        Slot::PORTABLE_TYPE_NAME
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        Cardinality::CARDINALITY
    }

    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        ApplicationQueryResultSlotKey::relation(
            Query::QUERY_TYPE_IDENTITY,
            Slot::PORTABLE_TYPE_IDENTITY,
            ApplicationQueryResultRelationSlotContract {
                relation: self.relation,
                from: self.from,
                to: self.to,
                direction: Direction::DIRECTION,
                output_name: self.output_name,
                cardinality: Cardinality::CARDINALITY,
            },
        )
    }
}

impl<Query, Slot, Schema, Relation, From, To>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        ForwardResultTraversal,
        OptionalOneResult,
    >
{
    pub fn forward_optional(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::new(output_name, relation)
    }
}

impl<Query, Slot, Schema, Relation, From, To>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        ForwardResultTraversal,
        ExactlyOneResult,
    >
{
    pub fn forward_one(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::new(output_name, relation)
    }
}

impl<Query, Slot, Schema, Relation, From, To>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        ForwardResultTraversal,
        ManyResults,
    >
{
    pub fn forward_many(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::new(output_name, relation)
    }
}

impl<Query, Slot, Schema, Relation, From, To>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        ReverseResultTraversal,
        OptionalOneResult,
    >
{
    pub fn reverse_optional(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::new(output_name, relation)
    }
}

impl<Query, Slot, Schema, Relation, From, To>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        ReverseResultTraversal,
        ExactlyOneResult,
    >
{
    pub fn reverse_one(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::new(output_name, relation)
    }
}

impl<Query, Slot, Schema, Relation, From, To>
    ApplicationQueryResultRelationRef<
        Query,
        Slot,
        Schema,
        Relation,
        From,
        To,
        ReverseResultTraversal,
        ManyResults,
    >
{
    pub fn reverse_many(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self {
        Self::new(output_name, relation)
    }
}
