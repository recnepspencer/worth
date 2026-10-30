//! Required child value whose entire installed Optional parent may be absent.
use super::result_slot_key::ApplicationQueryResultFieldSlotContract;
use super::{
    ApplicationQueryMarkerIdentity, ApplicationQueryResultFieldPresence,
    ApplicationQueryResultSlotKey,
};
use crate::application_schema::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationScalarValueBinding,
    RequiredApplicationFieldValue,
};
use crate::portable_identity::WorthQueryPortableType;
use std::marker::PhantomData;
use worth_foundational::facade::ScalarAspectType;
pub struct ApplicationQueryOptionalAspectFieldRef<
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
> {
    output_name: &'static str,
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
    _query_position: PhantomData<fn() -> (Query, Slot, Schema, Entity, Aspect)>,
    _field_contract: PhantomData<fn() -> (Field, Value, Write, Equality, Unit)>,
}
impl<Query, Slot, Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>
    ApplicationQueryOptionalAspectFieldRef<
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
    >
where
    Field: RequiredApplicationFieldValue<Value = Value>,
    Unit: ApplicationFieldUnit,
{
    pub fn new(
        output_name: &'static str,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self {
        Self {
            output_name,
            entity: field.entity(),
            aspect: field.aspect(),
            field: field.field(),
            _query_position: PhantomData,
            _field_contract: PhantomData,
        }
    }

    pub const fn output_name(&self) -> &'static str {
        self.output_name
    }

    pub const fn entity(&self) -> &'static str {
        self.entity
    }

    pub const fn aspect(&self) -> &'static str {
        self.aspect
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        Field::Binding::SCALAR_FAMILY
    }

    pub const fn presence(&self) -> ApplicationQueryResultFieldPresence {
        ApplicationQueryResultFieldPresence::OptionalAspect
    }

    pub fn value_type(&self) -> &'static str {
        Field::Binding::IDENTITY_NAME
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

    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        ApplicationQueryResultSlotKey::field(
            Query::QUERY_TYPE_IDENTITY,
            Slot::PORTABLE_TYPE_IDENTITY,
            ApplicationQueryResultFieldSlotContract {
                entity: self.entity,
                aspect: self.aspect,
                field: self.field,
                output_name: self.output_name,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                value_type: Field::Binding::IDENTITY,
                presence: ApplicationQueryResultFieldPresence::OptionalAspect,
            },
        )
    }
}

impl<Q, S, Schema, E, A, F, V, W, Eq, U> Copy
    for ApplicationQueryOptionalAspectFieldRef<Q, S, Schema, E, A, F, V, W, Eq, U>
{
}
impl<Q, S, Schema, E, A, F, V, W, Eq, U> Clone
    for ApplicationQueryOptionalAspectFieldRef<Q, S, Schema, E, A, F, V, W, Eq, U>
{
    fn clone(&self) -> Self {
        *self
    }
}
