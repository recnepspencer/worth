use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequest,
    ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, StringApplicationValueBinding,
};
use worth_query_declaration::{
    worth_query_capability, worth_query_operation, worth_query_operation_reads,
    worth_query_operation_writes,
};

use super::{
    CapabilityAction, CapabilityActionBinding, CapabilityElevationApprover,
    CapabilityElevationGrant, CapabilityElevationIdentity, CapabilityElevationNotAfter,
    CapabilityElevationNotBefore, CapabilityElevationReason, CapabilityElevationRequester,
    CapabilityElevationResource, CapabilityElevationReview, CapabilityElevationSlot,
    CapabilityElevationStatusField, CapabilityPurpose, CapabilityPurposeBinding,
    CapabilityRequestContext, CapabilityReviewIdentity, CapabilityReviewKindField,
    CapabilityReviewResource, CapabilityReviewStatusField, CapabilityReviewer,
};
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AccountIdentity, IdentityExecutionSchema,
};

worth_query_capability!(pub RevokeElevationCapability in IdentityExecutionSchema);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloseElevationInput {
    pub account: String,
    pub elevation: String,
}
worth_query_declaration::worth_query_portable_type!(
    CloseElevationInput => "worth.query.test.close-elevation-input.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub RevokeCapabilityElevationOperationInputBinding for CloseElevationInput { identity: "worth.query.test.close-elevation-input.v1" });
worth_query_operation!(pub RevokeCapabilityElevationOperation for IdentityExecutionSchema, input RevokeCapabilityElevationOperationInputBinding);
worth_query_operation_reads!(RevokeCapabilityElevationOperation => [CapabilityElevationIdentity, CapabilityElevationReason, CapabilityElevationStatusField, CapabilityElevationNotBefore, CapabilityElevationNotAfter, CapabilityReviewIdentity, CapabilityReviewKindField, CapabilityReviewStatusField, CapabilityElevationRequester, CapabilityElevationApprover, CapabilityElevationGrant, CapabilityElevationResource, CapabilityElevationReview, CapabilityReviewResource, CapabilityReviewer]);
worth_query_operation_writes!(RevokeCapabilityElevationOperation => [CapabilityElevationStatusField]);

impl ApplicationCapabilityRequest<IdentityExecutionSchema, RevokeElevationCapability>
    for CloseElevationInput
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
                CapabilityAction::RevokeElevation,
            )
            .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()).entity(
                CapabilityElevationSlot::reference(),
                ApplicationCapabilityEntitySelector::new(
                    CapabilityElevationIdentity::reference(),
                    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                        self.elevation.clone(),
                    )
                    .expect("fixture elevation identity must encode"),
                ),
            ),
        ))
    }
}
