use super::*;
use crate::application_capability::ApplicationCapabilityFieldBinding;
use crate::application_schema::{
    ApplicationFieldRef, EqualityPredicate, NoApplicationUnit, ReadOnly,
};

pub(super) struct Elevation;
pub(super) struct ElevationFacts;
pub(super) struct Review;
pub(super) struct ReviewFacts;
pub(super) struct ElevationIdentity;
pub(super) struct ElevationReason;
pub(super) struct ElevationStatus;
pub(super) struct ElevationNotBefore;
pub(super) struct ElevationNotAfter;
pub(super) struct ReviewIdentity;
pub(super) struct ReviewKind;
pub(super) struct ReviewStatus;
pub(super) struct Requester;
pub(super) struct Approver;
pub(super) struct ElevationGrant;
pub(super) struct ElevationResource;
pub(super) struct ElevationReview;
pub(super) struct ReviewScope;
pub(super) struct Reviewer;
pub(super) struct ElevationSlot;
pub(super) struct ReviewSlot;
pub(super) struct RequestOperation;
pub(super) struct ApproveOperation;
pub(super) struct RevokeOperation;
pub(super) struct CompleteReviewOperation;
pub(super) struct DuplicateApproveOperation;
pub(super) struct MissingRequestOperation;
pub(super) struct SwappedRequestOperation;
pub(super) struct SwappedApproveOperation;
pub(super) struct RequestCapability;
pub(super) struct ApproveCapability;
pub(super) struct RevokeCapability;
pub(super) struct CompleteReviewCapability;

declare_u64_field!(
    ElevationIdentity,
    ElevationReason,
    ElevationStatus,
    ElevationNotBefore,
    ElevationNotAfter,
    ReviewIdentity,
    ReviewKind,
    ReviewStatus,
);

macro_rules! operation_identity {
    ($operation:ty => $identifier:literal) => {
        impl ApplicationOperationMarkerIdentity<Schema> for $operation {
            type InputBinding = UnitOperationInputBinding;
            const IDENTIFIER: &'static str = $identifier;
        }
    };
}

operation_identity!(RequestOperation => "Request");
operation_identity!(ApproveOperation => "Approve");
operation_identity!(RevokeOperation => "Revoke");
operation_identity!(CompleteReviewOperation => "CompleteReview");
operation_identity!(DuplicateApproveOperation => "Request");
operation_identity!(MissingRequestOperation => "Missing");
operation_identity!(SwappedRequestOperation => "Approve");
operation_identity!(SwappedApproveOperation => "Request");

pub(super) fn elevation_value(value: u64) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(
        elevation_field::<ElevationStatus>("ElevationStatus"),
        encoded(value),
    )
}

pub(super) fn review_value(value: u64) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(
        review_field::<ReviewStatus>("ReviewStatus"),
        encoded(value),
    )
}

pub(super) fn elevation_binding<
    Field: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(elevation_field::<Field>(name))
}

pub(super) fn review_binding<
    Field: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(review_field::<Field>(name))
}

pub(super) fn elevation_field<
    Field: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationFieldRef<
    Schema,
    Elevation,
    ElevationFacts,
    Field,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationFieldRef::from_schema_identifiers("Elevation", "ElevationFacts", name)
}

pub(super) fn review_field<
    Field: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>,
>(
    name: &'static str,
) -> ApplicationFieldRef<
    Schema,
    Review,
    ReviewFacts,
    Field,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationFieldRef::from_schema_identifiers("Review", "ReviewFacts", name)
}

pub(super) fn context_slot<Slot, Entity>(
    slot: &'static str,
    entity: &'static str,
) -> ApplicationCapabilityContextEntitySlotRef<Schema, Context, Slot, Entity> {
    ApplicationCapabilityContextEntitySlotRef::from_schema_identifiers(
        ApplicationCapabilityContextRef::from_schema_identifier("Context"),
        slot,
        ApplicationEntityRef::from_schema_identifier(entity),
    )
}

pub(super) fn operation<Marker>(_: &'static str) -> ApplicationOperationRef<Schema, Marker, ()>
where
    Marker: ApplicationOperationMarkerIdentity<Schema, InputBinding = UnitOperationInputBinding>,
{
    ApplicationOperationRef::from_declaration()
}

pub(super) fn transition_binding<CapabilityMarker, OperationMarker>(
    capability: &'static str,
    operation_name: &'static str,
) -> ApplicationCapabilityTransitionBinding
where
    OperationMarker:
        ApplicationOperationMarkerIdentity<Schema, InputBinding = UnitOperationInputBinding>,
{
    ApplicationCapabilityTransitionBinding::from_references(
        ApplicationCapabilityRef::<Schema, CapabilityMarker>::from_schema_identifier(capability),
        operation::<OperationMarker>(operation_name),
    )
}

pub(super) fn transition_contract<CapabilityMarker, OperationMarker>(
    capability: &'static str,
    operation_name: &'static str,
) -> crate::application_capability::ErasedApplicationCapabilityContract
where
    OperationMarker:
        ApplicationOperationMarkerIdentity<Schema, InputBinding = UnitOperationInputBinding>,
{
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, CapabilityMarker>::from_schema_identifier(capability),
        operation::<OperationMarker>(operation_name),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition())
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
    .erased()
    .clone()
}

fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
