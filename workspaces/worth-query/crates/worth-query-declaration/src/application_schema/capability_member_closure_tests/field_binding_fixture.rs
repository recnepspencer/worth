macro_rules! declare_u64_field {
    ($($field:ty),+ $(,)?) => {$ (
        impl crate::application_schema::DeclaredApplicationFieldValue for $field {
            type Value = u64;
            type Binding = crate::application_schema::U64ApplicationValueBinding;
            const PRESENCE: crate::application_schema::ApplicationFieldPresence =
                crate::application_schema::ApplicationFieldPresence::Required;
        }
    )+};
}

use super::{Facts, Grant, Resource, ResourceFacts, Schema};
use crate::application_capability::ApplicationCapabilityFieldBinding;
use crate::application_schema::{
    ApplicationFieldRef, EqualityPredicate, NoApplicationUnit, ReadOnly,
};

pub(super) fn field<
    FieldMarker: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationFieldRef<
    Schema,
    Grant,
    Facts,
    FieldMarker,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationFieldRef::from_schema_identifiers("Grant", "Facts", name)
}

pub(super) fn binding<
    FieldMarker: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(field::<FieldMarker>(name))
}

pub(super) fn resource_binding<
    FieldMarker: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(ApplicationFieldRef::<
        Schema,
        Resource,
        ResourceFacts,
        FieldMarker,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >::from_schema_identifiers(
        "Resource", "ResourceFacts", name
    ))
}

pub(super) fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
