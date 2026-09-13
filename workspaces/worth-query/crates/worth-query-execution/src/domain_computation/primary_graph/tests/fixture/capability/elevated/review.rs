use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequest,
    ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, StringApplicationValueBinding,
};
use worth_query_declaration::{
    worth_query_capability, worth_query_operation, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_writes,
};

use super::{
    CapabilityAction, CapabilityActionBinding, CapabilityElevationApprover,
    CapabilityElevationGrant, CapabilityElevationIdentity, CapabilityElevationNotAfter,
    CapabilityElevationNotBefore, CapabilityElevationReason, CapabilityElevationRequester,
    CapabilityElevationResource, CapabilityElevationReview, CapabilityElevationSlot,
    CapabilityElevationStatusField, CapabilityPurpose, CapabilityPurposeBinding,
    CapabilityRequestContext, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewSlot, CapabilityReviewStatusField,
    CapabilityReviewer,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AccountIdentity, IdentityExecutionSchema,
};

worth_query_capability!(pub CompleteElevationReviewCapability in IdentityExecutionSchema);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteElevationReviewInput {
    pub account: String,
    pub elevation: String,
    pub review: String,
}
worth_query_declaration::worth_query_portable_type!(
    CompleteElevationReviewInput => "worth.query.test.complete-elevation-review-input.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub CompleteCapabilityReviewOperationInputBinding for CompleteElevationReviewInput { identity: "worth.query.test.complete-elevation-review-input.v1" });
worth_query_operation!(pub CompleteCapabilityReviewOperation for IdentityExecutionSchema, input CompleteCapabilityReviewOperationInputBinding);
worth_query_operation_reads!(CompleteCapabilityReviewOperation => [CapabilityElevationIdentity, CapabilityElevationReason, CapabilityElevationStatusField, CapabilityElevationNotBefore, CapabilityElevationNotAfter, CapabilityReviewIdentity, CapabilityReviewKindField, CapabilityReviewStatusField, CapabilityElevationRequester, CapabilityElevationApprover, CapabilityElevationGrant, CapabilityElevationResource, CapabilityElevationReview, CapabilityReviewResource, CapabilityReviewer]);
worth_query_operation_writes!(CompleteCapabilityReviewOperation => [CapabilityReviewStatusField]);
worth_query_operation_links!(CompleteCapabilityReviewOperation => [CapabilityReviewer]);

impl ApplicationCapabilityRequest<IdentityExecutionSchema, CompleteElevationReviewCapability>
    for CompleteElevationReviewInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::CompleteReview,
            )
            .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference())
                .entity(
                    CapabilityElevationSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        CapabilityElevationIdentity::reference(),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.elevation.clone(),
                        )
                        .expect("fixture elevation identity must encode"),
                    ),
                )
                .entity(
                    CapabilityReviewSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        CapabilityReviewIdentity::reference(),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.review.clone(),
                        )
                        .expect("fixture review identity must encode"),
                    ),
                ),
        ))
    }
}
