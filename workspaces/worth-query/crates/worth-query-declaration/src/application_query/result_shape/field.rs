use worth_foundational::facade::ScalarAspectType;

use crate::application_query::ApplicationQueryResultFieldPresence;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::super::{
    result_slot_key::ApplicationQueryResultFieldSlotContract, ApplicationQueryResultSlotKey,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryResultField {
    slot_key: ApplicationQueryResultSlotKey,
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    entity: String,
    aspect: String,
    field: String,
    output_name: String,
    scalar_family: ScalarAspectType,
    value_type: WorthQueryPortableTypeIdentity,
    presence: ApplicationQueryResultFieldPresence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryResultFieldParts {
    pub query_type: WorthQueryPortableTypeIdentity,
    pub slot_type: WorthQueryPortableTypeIdentity,
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub output_name: String,
    pub scalar_family: ScalarAspectType,
    pub value_type: WorthQueryPortableTypeIdentity,
    pub presence: ApplicationQueryResultFieldPresence,
}

impl ApplicationQueryResultField {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryResultFieldParts) -> Self {
        let slot_key = ApplicationQueryResultSlotKey::field(
            parts.query_type.clone(),
            parts.slot_type.clone(),
            ApplicationQueryResultFieldSlotContract {
                entity: &parts.entity,
                aspect: &parts.aspect,
                field: &parts.field,
                output_name: &parts.output_name,
                scalar_family: parts.scalar_family,
                value_type: parts.value_type.clone(),
                presence: parts.presence,
            },
        );
        Self {
            slot_key,
            query_type: parts.query_type,
            slot_type: parts.slot_type,
            entity: parts.entity,
            aspect: parts.aspect,
            field: parts.field,
            output_name: parts.output_name,
            scalar_family: parts.scalar_family,
            value_type: parts.value_type,
            presence: parts.presence,
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

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    pub const fn value_type(&self) -> &str {
        self.value_type.as_str()
    }

    pub fn value_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.value_type.clone()
    }

    pub const fn presence(&self) -> ApplicationQueryResultFieldPresence {
        self.presence
    }
}
