use super::*;
use crate::application_capability::ApplicationCapabilityFieldBinding;

pub(super) struct Elevation;
pub(super) struct ElevationFacts;
pub(super) struct Review;
pub(super) struct ReviewFacts;
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
    ApplicationCapabilityValueBinding::new(elevation_field("ElevationStatus"), encoded(value))
        .unwrap()
}

pub(super) fn review_value(value: u64) -> ApplicationCapabilityValueBinding {
    ApplicationCapabilityValueBinding::new(review_field("ReviewStatus"), encoded(value)).unwrap()
}

pub(super) fn elevation_field(
    name: &'static str,
) -> fragment::FieldSelection<ElevationModel, Elevation, ElevationFacts, u64> {
    fragment::FieldSelection::at(name)
}
pub(super) fn elevation_binding(name: &'static str) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(elevation_field(name)).unwrap()
}
pub(super) fn review_field(
    name: &'static str,
) -> fragment::FieldSelection<ElevationModel, Review, ReviewFacts, u64> {
    fragment::FieldSelection::at(name)
}
pub(super) fn review_binding(name: &'static str) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(review_field(name)).unwrap()
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

use crate::application_schema::{fragment, U64ApplicationValueBinding, ValueDecodeAdmission};
pub(super) struct ElevationModel;
impl fragment::Fragment for ElevationModel {
    const NAME: &'static str = "ElevationModel";
    type Bindings = ElevationBindings;
    type Members = (Elevation, Review);
}
pub(super) struct ElevationBindings;
impl fragment::BindingSet for ElevationBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for ElevationBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
impl fragment::Entity for Elevation {
    const NAME: &'static str = "Elevation";
    type Aspects = (ElevationFacts,);
    type Relations = ();
}
impl fragment::Aspect for ElevationFacts {
    const NAME: &'static str = "ElevationFacts";
    type Row = ElevationRow;
}
pub(super) struct ElevationRow(u64, u64, u64, u64, u64, u64);
impl fragment::Record<ElevationRow> for ElevationBindings {
    fn record() -> impl fragment::Shape<ElevationRow, Self> {
        fragment::field("ElevationStatus", |row: &ElevationRow| row.0)
            .and("ElevationClosedAt", |row: &ElevationRow| row.1)
            .and("ElevationIdentity", |row: &ElevationRow| row.2)
            .and("ElevationNotAfter", |row: &ElevationRow| row.3)
            .and("ElevationNotBefore", |row: &ElevationRow| row.4)
            .and("ElevationReason", |row: &ElevationRow| row.5)
            .build(|v0, v1, v2, v3, v4, v5, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ElevationRow(v0, v1, v2, v3, v4, v5))
            })
    }
}
impl fragment::Entity for Review {
    const NAME: &'static str = "Review";
    type Aspects = (ReviewFacts,);
    type Relations = ();
}
impl fragment::Aspect for ReviewFacts {
    const NAME: &'static str = "ReviewFacts";
    type Row = ReviewRow;
}
pub(super) struct ReviewRow(u64, u64, u64, u64);
impl fragment::Record<ReviewRow> for ElevationBindings {
    fn record() -> impl fragment::Shape<ReviewRow, Self> {
        fragment::field("ReviewKind", |row: &ReviewRow| row.0)
            .and("ReviewStatus", |row: &ReviewRow| row.1)
            .and("ReviewIdentity", |row: &ReviewRow| row.2)
            .and("ReviewReviewedAt", |row: &ReviewRow| row.3)
            .build(|v0, v1, v2, v3, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ReviewRow(v0, v1, v2, v3))
            })
    }
}
