//! Query result selectors borrow record-derived names; schema build checks the requested column.
use super::result_slot_key::ApplicationQueryResultFieldSlotContract;
use super::{
    ApplicationQueryMarkerIdentity, ApplicationQueryResultFieldPresence,
    ApplicationQueryResultSlotKey,
};
use crate::application_schema::{
    fragment::RecordSelection, ApplicationFieldBindingRecipe, ApplicationScalarValueBinding,
};
use crate::portable_identity::WorthQueryPortableType;
use std::marker::PhantomData;
use worth_foundational::facade::ScalarAspectType;

/// An authored Required projection; installation retains its checked slot contract.
pub struct ApplicationQueryResultFieldRef<Query, Slot, Schema, Field: RecordSelection> {
    output_name: &'static str,
    source: Field,
    marker: PhantomData<fn() -> (Query, Slot, Schema)>,
}
impl<Query, Slot, Schema, Field: RecordSelection> Copy
    for ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>
{
}
impl<Query, Slot, Schema, Field: RecordSelection> Clone
    for ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<Query, Slot, Schema, Field: RecordSelection>
    ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>
{
    pub fn new(output_name: &'static str, field: Field) -> Self {
        Self {
            output_name,
            source: field,
            marker: PhantomData,
        }
    }
    pub const fn output_name(&self) -> &'static str {
        self.output_name
    }
    pub fn entity(&self) -> &'static str {
        self.source.entity()
    }
    pub fn aspect(&self) -> &'static str {
        self.source.aspect()
    }
    pub fn field(&self) -> &'static str {
        self.source.field()
    }
    pub const fn scalar_family(&self) -> ScalarAspectType {
        Field::Binding::SCALAR_FAMILY
    }
    pub const fn presence(&self) -> ApplicationQueryResultFieldPresence {
        ApplicationQueryResultFieldPresence::Required
    }
    pub fn value_type(&self) -> &'static str {
        Field::Binding::IDENTITY_NAME
    }
    pub(crate) fn binding_recipe(&self) -> ApplicationFieldBindingRecipe {
        self.source.expected_recipe()
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
    pub fn matches_slot_key(&self, key: &ApplicationQueryResultSlotKey) -> bool
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        key.matches_field(
            Query::QUERY_TYPE_IDENTITY,
            Slot::PORTABLE_TYPE_IDENTITY,
            self.contract(),
        )
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        ApplicationQueryResultSlotKey::field(
            Query::QUERY_TYPE_IDENTITY,
            Slot::PORTABLE_TYPE_IDENTITY,
            self.contract(),
        )
    }
    fn contract(&self) -> ApplicationQueryResultFieldSlotContract<'_> {
        ApplicationQueryResultFieldSlotContract {
            entity: self.entity(),
            aspect: self.aspect(),
            field: self.field(),
            output_name: self.output_name,
            scalar_family: Field::Binding::SCALAR_FAMILY,
            value_type: Field::Binding::IDENTITY,
            presence: self.presence(),
        }
    }
}

/// An authored Optional projection; installation retains its checked slot contract.
pub struct ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field: RecordSelection> {
    output_name: &'static str,
    source: Field,
    marker: PhantomData<fn() -> (Query, Slot, Schema)>,
}
impl<Query, Slot, Schema, Field: RecordSelection> Copy
    for ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>
{
}
impl<Query, Slot, Schema, Field: RecordSelection> Clone
    for ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>
{
    fn clone(&self) -> Self {
        *self
    }
}
impl<Query, Slot, Schema, Field: RecordSelection>
    ApplicationQueryOptionalResultFieldRef<Query, Slot, Schema, Field>
{
    pub fn new(output_name: &'static str, field: Field) -> Self {
        Self {
            output_name,
            source: field,
            marker: PhantomData,
        }
    }
    pub const fn output_name(&self) -> &'static str {
        self.output_name
    }
    pub fn entity(&self) -> &'static str {
        self.source.entity()
    }
    pub fn aspect(&self) -> &'static str {
        self.source.aspect()
    }
    pub fn field(&self) -> &'static str {
        self.source.field()
    }
    pub const fn scalar_family(&self) -> ScalarAspectType {
        Field::Binding::SCALAR_FAMILY
    }
    pub const fn presence(&self) -> ApplicationQueryResultFieldPresence {
        ApplicationQueryResultFieldPresence::Optional
    }
    pub fn value_type(&self) -> &'static str {
        Field::Binding::IDENTITY_NAME
    }
    pub(crate) fn binding_recipe(&self) -> ApplicationFieldBindingRecipe {
        self.source.expected_recipe()
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
    pub fn matches_slot_key(&self, key: &ApplicationQueryResultSlotKey) -> bool
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        key.matches_field(
            Query::QUERY_TYPE_IDENTITY,
            Slot::PORTABLE_TYPE_IDENTITY,
            self.contract(),
        )
    }
    pub fn slot_key(&self) -> ApplicationQueryResultSlotKey
    where
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        ApplicationQueryResultSlotKey::field(
            Query::QUERY_TYPE_IDENTITY,
            Slot::PORTABLE_TYPE_IDENTITY,
            self.contract(),
        )
    }
    fn contract(&self) -> ApplicationQueryResultFieldSlotContract<'_> {
        ApplicationQueryResultFieldSlotContract {
            entity: self.entity(),
            aspect: self.aspect(),
            field: self.field(),
            output_name: self.output_name,
            scalar_family: Field::Binding::SCALAR_FAMILY,
            value_type: Field::Binding::IDENTITY,
            presence: self.presence(),
        }
    }
}
