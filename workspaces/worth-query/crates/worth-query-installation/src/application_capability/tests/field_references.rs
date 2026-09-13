//! Marker-bound field references used by capability identity tests.

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityFieldBinding,
    application_schema::{
        ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
        ApplicationFieldMarkerIdentity, ApplicationFieldPresence, ApplicationFieldRef,
        DeclaredApplicationFieldValue, EqualityPredicate, NoApplicationUnit, ReadOnly,
        U64ApplicationValueBinding,
    },
};

use super::{
    Action, Amount, ChangedResourceWorkflow, ChangedValidFrom, ChangedWorkflow, DelegationLimit,
    Facts, Field, Grant, Principal, Purpose, Resource, ResourceFacts, ResourceWorkflow, Schema,
    Status, ValidFrom, ValidThrough, Workflow,
};

macro_rules! entity_identity {
    ($marker:ty, $identifier:literal) => {
        impl ApplicationEntityMarkerIdentity<Schema> for $marker {
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

macro_rules! aspect_identity {
    ($marker:ty, $entity:ty, $identifier:literal, $identity:expr) => {
        impl ApplicationAspectMarkerIdentity<Schema, $entity> for $marker {
            const IDENTIFIER: &'static str = $identifier;
            const ASPECT_IDENTITY:
                worth_query_declaration::facade::application_schema::AspectIdentity =
                worth_query_declaration::facade::application_schema::AspectIdentity($identity);
            const CONTRACT_REVISION:
                worth_query_declaration::facade::application_schema::AspectContractRevision =
                worth_query_declaration::facade::application_schema::AspectContractRevision(1);
        }
    };
}

macro_rules! field_identity {
    ($marker:ty, $entity:ty, $aspect:ty, $identifier:literal) => {
        impl ApplicationFieldMarkerIdentity<Schema, $entity, $aspect> for $marker {
            const IDENTIFIER: &'static str = $identifier;
        }
        impl DeclaredApplicationFieldValue for $marker {
            type Value = u64;
            type Binding = U64ApplicationValueBinding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        }
    };
}

entity_identity!(Grant, "Grant");
entity_identity!(Resource, "Resource");
entity_identity!(Principal, "Principal");
aspect_identity!(Facts, Grant, "Facts", 0x9161_2201);
aspect_identity!(ResourceFacts, Resource, "ResourceFacts", 0x9161_2202);
field_identity!(Action, Grant, Facts, "Action");
field_identity!(Purpose, Grant, Facts, "Purpose");
field_identity!(Field, Grant, Facts, "Field");
field_identity!(Amount, Grant, Facts, "Amount");
field_identity!(Workflow, Grant, Facts, "Workflow");
field_identity!(ChangedWorkflow, Grant, Facts, "ChangedWorkflow");
field_identity!(
    ResourceWorkflow,
    Resource,
    ResourceFacts,
    "ResourceWorkflow"
);
field_identity!(
    ChangedResourceWorkflow,
    Resource,
    ResourceFacts,
    "ChangedResourceWorkflow"
);
field_identity!(Status, Grant, Facts, "Status");
field_identity!(ValidFrom, Grant, Facts, "ValidFrom");
field_identity!(ChangedValidFrom, Grant, Facts, "ChangedValidFrom");
field_identity!(ValidThrough, Grant, Facts, "ValidThrough");
field_identity!(DelegationLimit, Grant, Facts, "DelegationLimit");

pub(super) fn field<FieldMarker>() -> ApplicationFieldRef<
    Schema,
    Grant,
    Facts,
    FieldMarker,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>
where
    FieldMarker: ApplicationFieldMarkerIdentity<Schema, Grant, Facts, Value = u64>,
{
    ApplicationFieldRef::from_schema_types()
}

pub(super) fn field_binding<FieldMarker>() -> ApplicationCapabilityFieldBinding
where
    FieldMarker: ApplicationFieldMarkerIdentity<Schema, Grant, Facts, Value = u64>,
{
    ApplicationCapabilityFieldBinding::from_reference(field::<FieldMarker>())
}

pub(super) fn resource_field_binding<FieldMarker>() -> ApplicationCapabilityFieldBinding
where
    FieldMarker: ApplicationFieldMarkerIdentity<Schema, Resource, ResourceFacts, Value = u64>,
{
    ApplicationCapabilityFieldBinding::from_reference(ApplicationFieldRef::<
        Schema,
        Resource,
        ResourceFacts,
        FieldMarker,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >::from_schema_types())
}

pub(super) fn resource_field<FieldMarker>() -> ApplicationFieldRef<
    Schema,
    Resource,
    ResourceFacts,
    FieldMarker,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>
where
    FieldMarker: ApplicationFieldMarkerIdentity<Schema, Resource, ResourceFacts, Value = u64>,
{
    ApplicationFieldRef::from_schema_types()
}
