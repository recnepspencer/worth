use worth_foundational::facade::ScalarAspectType;

use crate::application_schema::{
    ApplicationFieldUnit, ApplicationScalarValueBinding, RequiredApplicationFieldValue,
};

use super::{ApplicationQueryMarkerIdentity, ApplicationQueryResultFieldRef};
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryOrderingDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryOrderingTerm {
    query_type: WorthQueryPortableTypeIdentity,
    slot_type: WorthQueryPortableTypeIdentity,
    entity: String,
    aspect: String,
    field: String,
    output_name: String,
    scalar_family: ScalarAspectType,
    value_type: WorthQueryPortableTypeIdentity,
    direction: ApplicationQueryOrderingDirection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryOrderingParts {
    pub query_type: WorthQueryPortableTypeIdentity,
    pub slot_type: WorthQueryPortableTypeIdentity,
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub output_name: String,
    pub scalar_family: ScalarAspectType,
    pub value_type: WorthQueryPortableTypeIdentity,
    pub direction: ApplicationQueryOrderingDirection,
}

impl ApplicationQueryOrderingTerm {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryOrderingParts) -> Self {
        Self {
            query_type: parts.query_type,
            slot_type: parts.slot_type,
            entity: parts.entity,
            aspect: parts.aspect,
            field: parts.field,
            output_name: parts.output_name,
            scalar_family: parts.scalar_family,
            value_type: parts.value_type,
            direction: parts.direction,
        }
    }

    pub(super) fn from_result_field<
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
    >(
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
        direction: ApplicationQueryOrderingDirection,
    ) -> Self
    where
        Field: RequiredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        Self {
            query_type: selector.slot_key().query_identity(),
            slot_type: selector.slot_key().slot_identity(),
            entity: selector.entity().to_owned(),
            aspect: selector.aspect().to_owned(),
            field: selector.field().to_owned(),
            output_name: selector.output_name().to_owned(),
            scalar_family: selector.scalar_family(),
            value_type: Field::Binding::IDENTITY,
            direction,
        }
    }

    pub const fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub const fn slot_type(&self) -> &str {
        self.slot_type.as_str()
    }

    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, &self.aspect, &self.field)
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

    pub const fn direction(&self) -> ApplicationQueryOrderingDirection {
        self.direction
    }
}
