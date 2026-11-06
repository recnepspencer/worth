use worth_query_declaration::facade::application_capability::ApplicationCapabilityEntitySelector;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestContext;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjectionDenial;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::worth_query_capability;
use worth_query_declaration::worth_query_operation;
use worth_query_declaration::worth_query_operation_reads;
use worth_query_declaration::worth_query_operation_writes;

use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityActionBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationApprover;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationClosedAt;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationGrant;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationNotBefore;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReason;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationRequester;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationReview;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationSlot;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityElevationStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityPurposeBinding;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityRequestContext;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewIdentity;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewKindField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewResource;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatusField;
use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewer;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

worth_query_capability!(pub RevokeElevationCapability in IdentityExecutionSchema);

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CloseElevationInput {
    pub account: String,
    pub elevation: String,
}
worth_query_declaration::worth_query_portable_type!(
    CloseElevationInput => "worth.query.test.close-elevation-input.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(pub RevokeCapabilityElevationOperationInputBinding for CloseElevationInput { identity: "worth.query.test.close-elevation-input.v1" });
worth_query_operation!(pub RevokeCapabilityElevationOperation for IdentityExecutionSchema, input RevokeCapabilityElevationOperationInputBinding);
worth_query_operation_reads!(RevokeCapabilityElevationOperation => [CapabilityElevationIdentity, CapabilityElevationReason, CapabilityElevationStatusField, CapabilityElevationNotBefore, CapabilityReviewIdentity, CapabilityReviewKindField, CapabilityReviewStatusField, CapabilityElevationRequester, CapabilityElevationApprover, CapabilityElevationGrant, CapabilityElevationResource, CapabilityElevationReview, CapabilityReviewResource, CapabilityReviewer]);
worth_query_operation_writes!(RevokeCapabilityElevationOperation => [CapabilityElevationStatusField, CapabilityElevationClosedAt]);

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
                AccountIdentity::at("AccountIdentity"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            )
            .expect("the native capability field is declared"),
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
                    CapabilityElevationIdentity::at("CapabilityElevationIdentity"),
                    ApplicationEncodedScalarValue::<crate::domain_computation::primary_graph::tests::fixture::schema_types::storage_model::ElevationKeyBinding>::try_new(
                        self.elevation.clone().into(),
                    )
                    .expect("fixture elevation identity must encode"),
                )
                .expect("the native capability field is declared"),
            ),
        ))
    }
}
