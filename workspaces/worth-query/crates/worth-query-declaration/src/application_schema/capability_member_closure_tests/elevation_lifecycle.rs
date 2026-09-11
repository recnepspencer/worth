use super::*;
use crate::application_capability::{
    ApplicationCapabilityContextEntitySlotBinding, ApplicationCapabilityElevationDefinition,
    ApplicationCapabilityElevationLifecycleDefinition, ApplicationCapabilityElevationStates,
    ApplicationCapabilityMandatoryReviewDefinition, ApplicationCapabilityTransitionBinding,
};
use crate::application_schema::ApplicationFieldPresence;

#[path = "elevation_lifecycle/bindings.rs"]
mod bindings;
#[path = "elevation_lifecycle/canonical_identity.rs"]
mod canonical_identity;
#[path = "elevation_lifecycle/duration.rs"]
mod duration;
#[path = "elevation_lifecycle/emission.rs"]
mod emission;
#[path = "elevation_lifecycle/member_fixture.rs"]
mod member_fixture;
#[path = "elevation_lifecycle/ownership.rs"]
mod ownership;

use bindings::*;
use member_fixture::*;

#[test]
fn governed_elevation_requires_one_closed_distinct_lifecycle() {
    let contract = elevation_contract(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::Distinct,
    );
    assert_eq!(build_from_members(elevation_members(contract)), Ok(()));
}

#[test]
fn duplicate_elevation_state_value_is_not_a_linear_lifecycle() {
    assert_eq!(
        build_from_members(elevation_members(elevation_contract(
            StatePosture::Duplicate,
            ReviewPosture::Distinct,
            LifecyclePosture::Distinct,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn duplicate_review_value_cannot_distinguish_required_from_completed() {
    assert_eq!(
        build_from_members(elevation_members(elevation_contract(
            StatePosture::Distinct,
            ReviewPosture::Duplicate,
            LifecyclePosture::Distinct,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn one_operation_cannot_own_two_lifecycle_roles() {
    assert_eq!(
        build_from_members(elevation_members(elevation_contract(
            StatePosture::Distinct,
            ReviewPosture::Distinct,
            LifecyclePosture::DuplicateOperation,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[test]
fn lifecycle_operation_must_exist_in_the_same_schema() {
    let contract = elevation_contract(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::MissingOperation,
    );
    assert_eq!(
        build_from_members(elevation_members(contract)),
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityDependency)
    );
}

#[test]
fn lifecycle_role_must_name_the_exact_installed_command_capability() {
    let contract = elevation_contract(
        StatePosture::Distinct,
        ReviewPosture::Distinct,
        LifecyclePosture::MissingCapability,
    );
    assert_eq!(
        build_from_members(elevation_members(contract)),
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationCapabilityDependency)
    );
}

#[test]
fn lifecycle_context_slots_must_name_the_exact_elevation_and_review_entities() {
    assert_eq!(
        build_from_members(elevation_members(elevation_contract(
            StatePosture::Distinct,
            ReviewPosture::Distinct,
            LifecyclePosture::WrongElevationSlot,
        ))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationCapability)
    );
}

#[derive(Clone, Copy)]
enum StatePosture {
    Distinct,
    Duplicate,
}

#[derive(Clone, Copy)]
enum ReviewPosture {
    Distinct,
    Duplicate,
}

#[derive(Clone, Copy)]
enum LifecyclePosture {
    Distinct,
    DuplicateOperation,
    MissingOperation,
    MissingCapability,
    WrongElevationSlot,
    SwappedOperations,
}

fn elevation_contract(
    states: StatePosture,
    review: ReviewPosture,
    lifecycle: LifecyclePosture,
) -> crate::application_capability::ErasedApplicationCapabilityContract {
    elevation_contract_with_duration(
        states,
        review,
        lifecycle,
        std::time::Duration::from_secs(20 * 60),
    )
}

fn elevation_contract_with_duration(
    states: StatePosture,
    review: ReviewPosture,
    lifecycle: LifecyclePosture,
    maximum_duration: std::time::Duration,
) -> crate::application_capability::ErasedApplicationCapabilityContract {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, Capability>::from_schema_identifier("Capability"),
        operation::<Operation>("Operation"),
        ApplicationEntityRef::<Schema, Grant>::from_schema_identifier("Grant"),
    )
    .target(target_definition(false, false))
    .constraints(constraint_definition())
    .delegation(delegation_definition())
    .composition(composition(true))
    .elevation(ApplicationCapabilityElevationRule::governed(
        elevation_definition(states, review, lifecycle, maximum_duration),
    ))
    .build()
    .erased()
    .clone()
}

fn elevation_definition(
    states: StatePosture,
    review: ReviewPosture,
    lifecycle: LifecyclePosture,
    maximum_duration: std::time::Duration,
) -> ApplicationCapabilityElevationDefinition {
    elevation_definition_with_lifecycle(
        states,
        review,
        maximum_duration,
        lifecycle_definition(lifecycle),
    )
}

fn elevation_definition_with_lifecycle(
    states: StatePosture,
    review: ReviewPosture,
    maximum_duration: std::time::Duration,
    lifecycle: ApplicationCapabilityElevationLifecycleDefinition,
) -> ApplicationCapabilityElevationDefinition {
    let values = match states {
        StatePosture::Distinct => [1, 2, 3, 4],
        StatePosture::Duplicate => [1, 2, 3, 3],
    };
    let completed = match review {
        ReviewPosture::Distinct => 2,
        ReviewPosture::Duplicate => 1,
    };
    ApplicationCapabilityElevationDefinition::new(
        elevation_binding::<ElevationIdentity>("ElevationIdentity"),
        elevation_binding::<ElevationReason>("ElevationReason"),
        elevation_binding::<ElevationStatus>("ElevationStatus"),
        ApplicationCapabilityElevationStates::new(
            elevation_value(values[0]),
            elevation_value(values[1]),
            elevation_value(values[2]),
            elevation_value(values[3]),
        ),
        ApplicationCapabilityValidityDefinition::new(
            ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
            elevation_binding::<ElevationNotBefore>("ElevationNotBefore"),
            elevation_binding::<ElevationNotAfter>("ElevationNotAfter"),
        ),
        maximum_duration,
        relation::<Requester, Principal, Elevation>("Requester", "Principal", "Elevation"),
        relation::<Approver, Principal, Elevation>("Approver", "Principal", "Elevation"),
        relation::<ElevationGrant, Elevation, Grant>("ElevationGrant", "Elevation", "Grant"),
        lifecycle,
        ApplicationCapabilityMandatoryReviewDefinition::new(
            relation::<ElevationReview, Elevation, Review>(
                "ElevationReview",
                "Elevation",
                "Review",
            ),
            review_binding::<ReviewIdentity>("ReviewIdentity"),
            ApplicationCapabilityValueBinding::new(
                review_field::<ReviewKind>("ReviewKind"),
                encoded(1_u64),
            ),
            relation::<ReviewScope, Review, Resource>("ReviewScope", "Review", "Resource"),
            relation::<Reviewer, Principal, Review>("Reviewer", "Principal", "Review"),
            review_binding::<ReviewStatus>("ReviewStatus"),
            review_value(1),
            review_value(completed),
        ),
    )
}

fn lifecycle_definition(
    posture: LifecyclePosture,
) -> ApplicationCapabilityElevationLifecycleDefinition {
    let request = match posture {
        LifecyclePosture::MissingOperation => transition_binding::<
            RequestCapability,
            MissingRequestOperation,
        >("RequestCapability", "Missing"),
        LifecyclePosture::SwappedOperations => transition_binding::<
            RequestCapability,
            SwappedRequestOperation,
        >("RequestCapability", "Approve"),
        LifecyclePosture::Distinct
        | LifecyclePosture::DuplicateOperation
        | LifecyclePosture::WrongElevationSlot
        | LifecyclePosture::MissingCapability => {
            transition_binding::<RequestCapability, RequestOperation>(
                if matches!(posture, LifecyclePosture::MissingCapability) {
                    "MissingCapability"
                } else {
                    "RequestCapability"
                },
                "Request",
            )
        }
    };
    let approve = match posture {
        LifecyclePosture::DuplicateOperation => transition_binding::<
            ApproveCapability,
            DuplicateApproveOperation,
        >("ApproveCapability", "Request"),
        LifecyclePosture::SwappedOperations => transition_binding::<
            ApproveCapability,
            SwappedApproveOperation,
        >("ApproveCapability", "Request"),
        LifecyclePosture::Distinct
        | LifecyclePosture::MissingOperation
        | LifecyclePosture::MissingCapability
        | LifecyclePosture::WrongElevationSlot => transition_binding::<
            ApproveCapability,
            ApproveOperation,
        >("ApproveCapability", "Approve"),
    };
    let elevation_slot = match posture {
        LifecyclePosture::WrongElevationSlot => {
            ApplicationCapabilityContextEntitySlotBinding::from_reference(resource_slot::<
                Context,
                ResourceSlot,
            >(
                "Context",
                "ResourceSlot",
            ))
        }
        _ => ApplicationCapabilityContextEntitySlotBinding::from_reference(context_slot::<
            ElevationSlot,
            Elevation,
        >(
            "ElevationSlot",
            "Elevation",
        )),
    };
    ApplicationCapabilityElevationLifecycleDefinition::new(
        elevation_slot,
        ApplicationCapabilityContextEntitySlotBinding::from_reference(context_slot::<
            ReviewSlot,
            Review,
        >(
            "ReviewSlot", "Review"
        )),
        request,
        approve,
        transition_binding::<RevokeCapability, RevokeOperation>("RevokeCapability", "Revoke"),
        transition_binding::<CompleteReviewCapability, CompleteReviewOperation>(
            "CompleteReviewCapability",
            "CompleteReview",
        ),
    )
}

fn elevation_members(
    contract: crate::application_capability::ErasedApplicationCapabilityContract,
) -> Vec<ApplicationSchemaMember> {
    let mut result = members(contract);
    result.extend([
        entity_member("Elevation"),
        entity_member("Review"),
        aspect_member("Elevation", "ElevationFacts"),
        aspect_member("Review", "ReviewFacts"),
        elevation_field_member("ElevationIdentity"),
        elevation_field_member("ElevationReason"),
        elevation_field_member("ElevationStatus"),
        elevation_field_member("ElevationNotBefore"),
        elevation_field_member("ElevationNotAfter"),
        review_field_member("ReviewIdentity"),
        review_field_member("ReviewKind"),
        review_field_member("ReviewStatus"),
        relation_member("Requester", "Principal", "Elevation"),
        relation_member("Approver", "Principal", "Elevation"),
        relation_member("ElevationGrant", "Elevation", "Grant"),
        relation_member("ElevationReview", "Elevation", "Review"),
        relation_member("ReviewScope", "Review", "Resource"),
        relation_member("Reviewer", "Principal", "Review"),
        context_slot_member("ElevationSlot", "ElevationSlot", "Elevation"),
        context_slot_member("ReviewSlot", "ReviewSlot", "Review"),
        operation_member("Request"),
        operation_member("Approve"),
        operation_member("Revoke"),
        operation_member("CompleteReview"),
        ApplicationSchemaMember::ApplicationCapability {
            contract: transition_contract::<RequestCapability, RequestOperation>(
                "RequestCapability",
                "Request",
            ),
        },
        ApplicationSchemaMember::ApplicationCapability {
            contract: transition_contract::<ApproveCapability, ApproveOperation>(
                "ApproveCapability",
                "Approve",
            ),
        },
        ApplicationSchemaMember::ApplicationCapability {
            contract: transition_contract::<RevokeCapability, RevokeOperation>(
                "RevokeCapability",
                "Revoke",
            ),
        },
        ApplicationSchemaMember::ApplicationCapability {
            contract: transition_contract::<CompleteReviewCapability, CompleteReviewOperation>(
                "CompleteReviewCapability",
                "CompleteReview",
            ),
        },
    ]);
    result
}

fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<
    crate::application_schema::U64ApplicationValueBinding,
> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
