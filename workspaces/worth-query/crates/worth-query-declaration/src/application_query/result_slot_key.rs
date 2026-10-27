use worth_foundational::facade::ScalarAspectType;

use super::{ApplicationQueryCardinality, ApplicationQueryResultTraversalDirection};
use crate::application_schema::ApplicationFieldPresence;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationQueryResultSlotKey {
    query: WorthQueryPortableTypeIdentity,
    slot: WorthQueryPortableTypeIdentity,
    contract: ApplicationQueryResultSlotContract,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ApplicationQueryResultSlotContract {
    /// Identity, full membership/length and anchor-forward order are one result.
    Orbit {
        relation: String,
        from: String,
        to: String,
        output_name: String,
    },
    Field {
        entity: String,
        aspect: String,
        field: String,
        output_name: String,
        scalar_family: ScalarAspectType,
        value_type: WorthQueryPortableTypeIdentity,
        presence: ApplicationFieldPresence,
    },
    Relation {
        relation: String,
        from: String,
        to: String,
        direction: ApplicationQueryResultTraversalDirection,
        output_name: String,
        cardinality: ApplicationQueryCardinality,
    },
}

pub(super) struct ApplicationQueryResultFieldSlotContract<'contract> {
    pub entity: &'contract str,
    pub aspect: &'contract str,
    pub field: &'contract str,
    pub output_name: &'contract str,
    pub scalar_family: ScalarAspectType,
    pub value_type: WorthQueryPortableTypeIdentity,
    pub presence: ApplicationFieldPresence,
}

pub(super) struct ApplicationQueryResultRelationSlotContract<'contract> {
    pub relation: &'contract str,
    pub from: &'contract str,
    pub to: &'contract str,
    pub direction: ApplicationQueryResultTraversalDirection,
    pub output_name: &'contract str,
    pub cardinality: ApplicationQueryCardinality,
}

impl ApplicationQueryResultSlotKey {
    pub(super) fn orbit(
        query: WorthQueryPortableTypeIdentity,
        slot: WorthQueryPortableTypeIdentity,
        relation: &str,
        from: &str,
        to: &str,
        output_name: &str,
    ) -> Self {
        Self {
            query,
            slot,
            contract: ApplicationQueryResultSlotContract::Orbit {
                relation: relation.into(),
                from: from.into(),
                to: to.into(),
                output_name: output_name.into(),
            },
        }
    }
    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.query.clone()
    }

    pub fn slot_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.slot.clone()
    }

    pub(super) fn field(
        query: WorthQueryPortableTypeIdentity,
        slot: WorthQueryPortableTypeIdentity,
        contract: ApplicationQueryResultFieldSlotContract<'_>,
    ) -> Self {
        Self {
            query,
            slot,
            contract: ApplicationQueryResultSlotContract::Field {
                entity: contract.entity.to_owned(),
                aspect: contract.aspect.to_owned(),
                field: contract.field.to_owned(),
                output_name: contract.output_name.to_owned(),
                scalar_family: contract.scalar_family,
                value_type: contract.value_type,
                presence: contract.presence,
            },
        }
    }

    pub(super) fn relation(
        query: WorthQueryPortableTypeIdentity,
        slot: WorthQueryPortableTypeIdentity,
        contract: ApplicationQueryResultRelationSlotContract<'_>,
    ) -> Self {
        Self {
            query,
            slot,
            contract: ApplicationQueryResultSlotContract::Relation {
                relation: contract.relation.to_owned(),
                from: contract.from.to_owned(),
                to: contract.to.to_owned(),
                direction: contract.direction,
                output_name: contract.output_name.to_owned(),
                cardinality: contract.cardinality,
            },
        }
    }
}
