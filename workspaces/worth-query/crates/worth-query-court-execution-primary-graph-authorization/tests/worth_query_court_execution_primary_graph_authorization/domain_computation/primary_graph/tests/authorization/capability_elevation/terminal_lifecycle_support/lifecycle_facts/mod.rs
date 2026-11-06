use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::*;

pub fn seal_lifecycle_fields<Operation>(
    reader: &mut worth_query_execution::facade::domain_computation::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
) {
    reader
        .require_decision_field(
            elevation,
            CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
        )
        .unwrap();
    reader
        .require_decision_field(
            elevation,
            CapabilityElevationReason::at("CapabilityElevationReason"),
        )
        .unwrap();
    reader
        .require_decision_field(
            elevation,
            CapabilityElevationStatusField::at("CapabilityElevationStatusField"),
        )
        .unwrap();
    reader
        .require_decision_field(
            elevation,
            CapabilityElevationNotBefore::at("CapabilityElevationNotBefore"),
        )
        .unwrap();
    reader
        .require_decision_field(
            elevation,
            CapabilityElevationNotAfter::at("CapabilityElevationNotAfter"),
        )
        .unwrap();
    reader
        .require_decision_field(
            review,
            CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
        )
        .unwrap();
    reader
        .require_decision_field(
            review,
            CapabilityReviewKindField::at("CapabilityReviewKindField"),
        )
        .unwrap();
    reader
        .require_decision_field(
            review,
            CapabilityReviewStatusField::at("CapabilityReviewStatusField"),
        )
        .unwrap();
}

pub fn seal_lifecycle_relations<Operation>(
    reader: &mut worth_query_execution::facade::domain_computation::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
) where
    CapabilityElevationRequester: OperationReads<Operation>,
    CapabilityElevationApprover: OperationReads<Operation>,
    CapabilityElevationGrant: OperationReads<Operation>,
    CapabilityElevationResource: OperationReads<Operation>,
    CapabilityElevationReview: OperationReads<Operation>,
    CapabilityReviewResource: OperationReads<Operation>,
    CapabilityReviewer: OperationReads<Operation>,
{
    reader
        .decision_relations_to(CapabilityElevationRequester::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_to(CapabilityElevationApprover::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationGrant::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationResource::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityElevationReview::reference(), elevation)
        .unwrap();
    reader
        .decision_relations_from(CapabilityReviewResource::reference(), review)
        .unwrap();
    reader
        .decision_relations_to(CapabilityReviewer::reference(), review)
        .unwrap();
}
