use super::*;

pub(super) fn seal_lifecycle_fields<Operation>(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
        IdentityExecutionSchema,
        Operation,
    >,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
) where
    CapabilityElevationIdentity: OperationReads<Operation>,
    CapabilityElevationReason: OperationReads<Operation>,
    CapabilityElevationStatusField: OperationReads<Operation>,
    CapabilityElevationNotBefore: OperationReads<Operation>,
    CapabilityElevationNotAfter: OperationReads<Operation>,
    CapabilityReviewIdentity: OperationReads<Operation>,
    CapabilityReviewKindField: OperationReads<Operation>,
    CapabilityReviewStatusField: OperationReads<Operation>,
{
    reader
        .require_decision_field(elevation, CapabilityElevationIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationReason::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationStatusField::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationNotBefore::reference())
        .unwrap();
    reader
        .require_decision_field(elevation, CapabilityElevationNotAfter::reference())
        .unwrap();
    reader
        .require_decision_field(review, CapabilityReviewIdentity::reference())
        .unwrap();
    reader
        .require_decision_field(review, CapabilityReviewKindField::reference())
        .unwrap();
    reader
        .require_decision_field(review, CapabilityReviewStatusField::reference())
        .unwrap();
}

pub(super) fn seal_lifecycle_relations<Operation>(
    reader: &mut crate::domain_computation::primary_graph::WorthQueryApplicationOperationInvariantProjectionReader<
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
