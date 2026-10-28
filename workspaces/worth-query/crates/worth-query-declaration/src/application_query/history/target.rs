use worth_foundational::facade::{AspectContractRevision, AspectIdentity};

use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::super::{ApplicationQueryCardinality, ApplicationQueryResultTraversalDirection};

/// Exact declaration selected by a history query; names are resolved at installation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationHistoryTarget {
    Entity {
        entity: String,
    },
    Relation {
        query: WorthQueryPortableTypeIdentity,
        slot: WorthQueryPortableTypeIdentity,
        relation: String,
        from: String,
        to: String,
        direction: ApplicationQueryResultTraversalDirection,
        cardinality: ApplicationQueryCardinality,
        output_name: String,
    },
}

impl ApplicationHistoryTarget {
    pub fn relation_slot_key(&self) -> Option<super::super::ApplicationQueryResultSlotKey> {
        let Self::Relation {
            query,
            slot,
            relation,
            from,
            to,
            direction,
            cardinality,
            output_name,
        } = self
        else {
            return None;
        };
        Some(super::super::ApplicationQueryResultSlotKey::relation(
            query.clone(),
            slot.clone(),
            super::super::result_slot_key::ApplicationQueryResultRelationSlotContract {
                relation,
                from,
                to,
                direction: *direction,
                output_name,
                cardinality: *cardinality,
            },
        ))
    }
}

/// A selected actual aspect contract, including its revision and record owner.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationHistoryAspectSelection {
    pub aspect: String,
    pub identity: AspectIdentity,
    pub revision: AspectContractRevision,
}
