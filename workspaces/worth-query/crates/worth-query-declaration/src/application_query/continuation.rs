use super::{
    ApplicationQueryCardinality, ApplicationQueryMarkerIdentity, ApplicationQueryResultRelationRef,
    ApplicationQueryResultTraversal, ApplicationQueryResultTraversalDirection, ManyResults,
};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

/// Identity-bearing declaration of the one result collection advanced by an
/// application-query continuation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryContinuationTarget {
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    relation: String,
    parent_entity: String,
    child_entity: String,
    direction: ApplicationQueryResultTraversalDirection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryContinuationParts {
    pub query_type: WorthQueryPortableTypeIdentity,
    pub slot_type: WorthQueryPortableTypeIdentity,
    pub relation: String,
    pub parent_entity: String,
    pub child_entity: String,
    pub direction: ApplicationQueryResultTraversalDirection,
}

impl ApplicationQueryContinuationTarget {
    pub fn from_untrusted_parts(
        parts: WorthQueryPortableApplicationQueryContinuationParts,
    ) -> Self {
        Self {
            query_type: parts.query_type,
            slot_type: parts.slot_type,
            relation: parts.relation,
            parent_entity: parts.parent_entity,
            child_entity: parts.child_entity,
            direction: parts.direction,
        }
    }

    pub(super) fn from_many_relation<Query, Slot, Schema, Relation, From, To, Direction>(
        relation: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ManyResults,
        >,
    ) -> Self
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        Self {
            query_type: relation.slot_key().query_identity(),
            slot_type: relation.slot_key().slot_identity(),
            relation: relation.relation().to_owned(),
            parent_entity: relation.parent().to_owned(),
            child_entity: relation.child().to_owned(),
            direction: relation.direction(),
        }
    }

    pub const fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.query_type.clone()
    }

    pub const fn slot_type(&self) -> &str {
        self.slot_type.as_str()
    }

    pub fn slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.slot_type.clone()
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn parent_entity(&self) -> &str {
        &self.parent_entity
    }

    pub fn child_entity(&self) -> &str {
        &self.child_entity
    }

    pub const fn direction(&self) -> ApplicationQueryResultTraversalDirection {
        self.direction
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        ApplicationQueryCardinality::Many
    }
}
