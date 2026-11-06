use worth_query_declaration::facade::application_capability::ApplicationCapabilityContextEntitySlotBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationLifecycleDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationRule;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityElevationStates;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityMandatoryReviewDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRelationBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityTransitionBinding;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityDefinition;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValueBinding;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;

use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ApproveCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::ApproveElevationCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationApprover;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationClosedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotAfter;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotBefore;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReason;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationRequester;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatus;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatusBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKind;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::value_bindings::CapabilityReviewKindBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKindField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewReviewedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatus;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewer;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CompleteCapabilityReviewOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CompleteElevationReviewCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RequestElevationCapability;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RevokeCapabilityElevationOperation;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::RevokeElevationCapability;

pub fn definition() -> ApplicationCapabilityElevationRule {
    let state = |value| {
        ApplicationCapabilityValueBinding::new(
            CapabilityElevationStatusField::at("CapabilityElevationStatusField"),
            ApplicationEncodedScalarValue::<CapabilityElevationStatusBinding>::try_new(value)
                .expect("fixture elevation status must encode"),
        )
        .unwrap()
    };
    ApplicationCapabilityElevationRule::governed(
        ApplicationCapabilityElevationDefinition::new(
            ApplicationCapabilityFieldBinding::from_reference(CapabilityElevationIdentity::at(
                "CapabilityElevationIdentity",
            ))
            .unwrap(),
            ApplicationCapabilityFieldBinding::from_reference(CapabilityElevationReason::at(
                "CapabilityElevationReason",
            ))
            .unwrap(),
            ApplicationCapabilityFieldBinding::from_reference(CapabilityElevationStatusField::at(
                "CapabilityElevationStatusField",
            ))
            .unwrap(),
            ApplicationCapabilityFieldBinding::from_reference(CapabilityElevationClosedAt::at(
                "CapabilityElevationClosedAt",
            ))
            .unwrap(),
            ApplicationCapabilityElevationStates::new(
                state(CapabilityElevationStatus::Requested),
                state(CapabilityElevationStatus::Approved),
                state(CapabilityElevationStatus::Expired),
                state(CapabilityElevationStatus::Revoked),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    CapabilityElevationNotBefore::at("CapabilityElevationNotBefore"),
                )
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityElevationNotAfter::at(
                    "CapabilityElevationNotAfter",
                ))
                .unwrap(),
            ),
            std::time::Duration::from_secs(20 * 60),
            ApplicationCapabilityRelationBinding::from_reference(
                CapabilityElevationRequester::reference(),
            ),
            ApplicationCapabilityRelationBinding::from_reference(
                CapabilityElevationApprover::reference(),
            ),
            ApplicationCapabilityRelationBinding::from_reference(
                CapabilityElevationGrant::reference(),
            ),
            ApplicationCapabilityElevationLifecycleDefinition::new(
                ApplicationCapabilityContextEntitySlotBinding::from_reference(
                    CapabilityElevationSlot::reference(),
                ),
                ApplicationCapabilityContextEntitySlotBinding::from_reference(
                    CapabilityReviewSlot::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references(
                    RequestElevationCapability::reference(),
                    RequestCapabilityElevationOperation::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references(
                    ApproveElevationCapability::reference(),
                    ApproveCapabilityElevationOperation::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references(
                    RevokeElevationCapability::reference(),
                    RevokeCapabilityElevationOperation::reference(),
                ),
                ApplicationCapabilityTransitionBinding::from_references(
                    CompleteElevationReviewCapability::reference(),
                    CompleteCapabilityReviewOperation::reference(),
                ),
            ),
            ApplicationCapabilityMandatoryReviewDefinition::new(
                ApplicationCapabilityRelationBinding::from_reference(
                    CapabilityElevationReview::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityReviewIdentity::at(
                    "CapabilityReviewIdentity",
                ))
                .unwrap(),
                ApplicationCapabilityValueBinding::new(
                    CapabilityReviewKindField::at("CapabilityReviewKindField"),
                    ApplicationEncodedScalarValue::<CapabilityReviewKindBinding>::try_new(
                        CapabilityReviewKind::Elevation,
                    )
                    .expect("fixture review kind must encode"),
                )
                .unwrap(),
                ApplicationCapabilityRelationBinding::from_reference(
                    CapabilityReviewResource::reference(),
                ),
                ApplicationCapabilityRelationBinding::from_reference(
                    CapabilityReviewer::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityReviewStatusField::at(
                    "CapabilityReviewStatusField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(CapabilityReviewReviewedAt::at(
                    "CapabilityReviewReviewedAt",
                ))
                .unwrap(),
                ApplicationCapabilityValueBinding::new(
                    CapabilityReviewStatusField::at("CapabilityReviewStatusField"),
                    ApplicationEncodedScalarValue::<CapabilityReviewStatusBinding>::try_new(
                        CapabilityReviewStatus::Required,
                    )
                    .expect("fixture review status must encode"),
                )
                .unwrap(),
                ApplicationCapabilityValueBinding::new(
                    CapabilityReviewStatusField::at("CapabilityReviewStatusField"),
                    ApplicationEncodedScalarValue::<CapabilityReviewStatusBinding>::try_new(
                        CapabilityReviewStatus::Completed,
                    )
                    .expect("fixture review status must encode"),
                )
                .unwrap(),
            ),
        )
        .with_resource_relation(ApplicationCapabilityRelationBinding::from_reference(
            CapabilityElevationResource::reference(),
        )),
    )
}
