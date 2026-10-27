//! A query-bound whole-orbit result slot. These names carry no read authority.
use super::{ApplicationQueryMarkerIdentity, ApplicationQueryResultSlotKey};
use crate::{
    application_schema::ApplicationRelationRef,
    portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity},
};
use std::marker::PhantomData;

/// Complete membership and persistent identity in Native anchor-forward order.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryOrbitMeaning {
    query: WorthQueryPortableTypeIdentity,
    slot: WorthQueryPortableTypeIdentity,
    relation: String,
    from: String,
    to: String,
    output_name: String,
}
/// Untrusted portable meaning; declaration and installed schema validation are mandatory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryOrbitParts {
    pub query: WorthQueryPortableTypeIdentity,
    pub slot: WorthQueryPortableTypeIdentity,
    pub relation: String,
    pub from: String,
    pub to: String,
    pub output_name: String,
}
impl ApplicationQueryOrbitMeaning {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryOrbitParts) -> Self {
        Self {
            query: parts.query,
            slot: parts.slot,
            relation: parts.relation,
            from: parts.from,
            to: parts.to,
            output_name: parts.output_name,
        }
    }
    pub const fn query_type(&self) -> &str {
        self.query.as_str()
    }
    pub const fn slot_type(&self) -> &str {
        self.slot.as_str()
    }
    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.query.clone()
    }
    pub fn slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.slot.clone()
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
    pub fn output_name(&self) -> &str {
        &self.output_name
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        ApplicationQueryResultSlotKey::orbit(
            self.query.clone(),
            self.slot.clone(),
            &self.relation,
            &self.from,
            &self.to,
            &self.output_name,
        )
    }
}
/// Select the declared whole-orbit slot in this query's complete result.
pub struct ApplicationQueryOrbitRef<Schema, Query, Slot, Relation, Entity> {
    meaning: ApplicationQueryOrbitMeaning,
    _marker: PhantomData<fn() -> (Schema, Query, Slot, Relation, Entity)>,
}
impl<Schema, Query, Slot, Relation, Entity>
    ApplicationQueryOrbitRef<Schema, Query, Slot, Relation, Entity>
where
    Query: ApplicationQueryMarkerIdentity<Schema, Scope = Entity>,
    Slot: WorthQueryPortableType,
{
    pub fn new(
        output_name: &'static str,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
    ) -> Self {
        Self {
            meaning: ApplicationQueryOrbitMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationQueryOrbitParts {
                    query: WorthQueryPortableTypeIdentity::declared(Query::QUERY_TYPE_NAME),
                    slot: Slot::PORTABLE_TYPE_IDENTITY,
                    relation: relation.name().into(),
                    from: relation.from().into(),
                    to: relation.to().into(),
                    output_name: output_name.into(),
                },
            ),
            _marker: PhantomData,
        }
    }
}
impl<Schema, Query, Slot, Relation, Entity>
    ApplicationQueryOrbitRef<Schema, Query, Slot, Relation, Entity>
{
    pub fn meaning(&self) -> &ApplicationQueryOrbitMeaning {
        &self.meaning
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey {
        self.meaning.slot_key()
    }
    pub(crate) fn into_meaning(self) -> ApplicationQueryOrbitMeaning {
        self.meaning
    }
}
