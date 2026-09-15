use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::super::{
    result_slot_key::ApplicationQueryResultRelationSlotContract, ApplicationQueryCardinality,
    ApplicationQueryPredicate, ApplicationQueryResultSlotKey,
    ApplicationQueryResultTraversalDirection,
};
use super::ApplicationQueryResultShape;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryResultRelation {
    slot_key: ApplicationQueryResultSlotKey,
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    relation: String,
    from: String,
    to: String,
    direction: ApplicationQueryResultTraversalDirection,
    output_name: String,
    cardinality: ApplicationQueryCardinality,
    predicate: Option<ApplicationQueryPredicate>,
    nested_shape: Box<ApplicationQueryResultShape>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryResultRelationParts {
    pub query_type: WorthQueryPortableTypeIdentity,
    pub slot_type: WorthQueryPortableTypeIdentity,
    pub relation: String,
    pub from: String,
    pub to: String,
    pub direction: ApplicationQueryResultTraversalDirection,
    pub output_name: String,
    pub cardinality: ApplicationQueryCardinality,
    pub predicate: Option<ApplicationQueryPredicate>,
    pub nested_shape: ApplicationQueryResultShape,
}

impl ApplicationQueryResultRelation {
    pub fn from_untrusted_parts(
        parts: WorthQueryPortableApplicationQueryResultRelationParts,
    ) -> Self {
        let slot_key = ApplicationQueryResultSlotKey::relation(
            parts.query_type.clone(),
            parts.slot_type.clone(),
            ApplicationQueryResultRelationSlotContract {
                relation: &parts.relation,
                from: &parts.from,
                to: &parts.to,
                direction: parts.direction,
                output_name: &parts.output_name,
                cardinality: parts.cardinality,
            },
        );
        Self {
            slot_key,
            query_type: parts.query_type,
            slot_type: parts.slot_type,
            relation: parts.relation,
            from: parts.from,
            to: parts.to,
            direction: parts.direction,
            output_name: parts.output_name,
            cardinality: parts.cardinality,
            predicate: parts.predicate,
            nested_shape: Box::new(parts.nested_shape),
        }
    }

    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        self.slot_key.clone()
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

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> ApplicationQueryResultTraversalDirection {
        self.direction
    }

    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        self.cardinality
    }

    pub fn predicate(&self) -> Option<&ApplicationQueryPredicate> {
        self.predicate.as_ref()
    }

    pub fn nested_shape(&self) -> &ApplicationQueryResultShape {
        &self.nested_shape
    }
}
