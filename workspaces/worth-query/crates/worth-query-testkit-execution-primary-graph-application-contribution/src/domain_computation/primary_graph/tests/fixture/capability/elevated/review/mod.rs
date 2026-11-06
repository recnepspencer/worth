use worth_query_declaration::facade::application_capability::ApplicationCapabilityEntitySelector;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestContext;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjectionDenial;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::worth_query_capability;
use worth_query_declaration::worth_query_operation;
use worth_query_declaration::worth_query_operation_links;
use worth_query_declaration::worth_query_operation_reads;

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityActionBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationApprover;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationRequester;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityPurposeBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityRequestContext;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewer;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

worth_query_capability!(pub CompleteElevationReviewCapability in IdentityExecutionSchema);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
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
worth_query_operation_reads!(CompleteCapabilityReviewOperation => [CapabilityElevationRequester, CapabilityElevationApprover, CapabilityElevationGrant, CapabilityElevationResource, CapabilityElevationReview, CapabilityReviewResource, CapabilityReviewer]);

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
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ).unwrap(),
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
                        CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
                        ApplicationEncodedScalarValue::<<CapabilityElevationIdentity as worth_query_declaration::facade::application_schema::fragment::RecordSelection>::Binding>::try_new(self.elevation.clone().into())
                        .expect("fixture elevation identity must encode"),
                    ).unwrap(),
                )
                .entity(
                    CapabilityReviewSlot::reference(),
                    ApplicationCapabilityEntitySelector::new(
                        CapabilityReviewIdentity::at("CapabilityReviewIdentity"),
                        ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                            self.review.clone(),
                        )
                        .expect("fixture review identity must encode"),
                    ).unwrap(),
                ),
        ))
    }
}
