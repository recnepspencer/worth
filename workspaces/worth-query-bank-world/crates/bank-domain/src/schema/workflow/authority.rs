#[path = "authority/records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_decl::facade::{
    application_capability::{
        ApplicationCapabilityActorComposition, ApplicationCapabilityAllowRule,
        ApplicationCapabilityCardinalityDimension, ApplicationCapabilityComposition,
        ApplicationCapabilityConflictRule, ApplicationCapabilityConstraintDefinition,
        ApplicationCapabilityContract, ApplicationCapabilityContractBuilder,
        ApplicationCapabilityCurrentnessDefinition, ApplicationCapabilityDecisionComposition,
        ApplicationCapabilityDelegationDefinition, ApplicationCapabilityDelegationRule,
        ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
        ApplicationCapabilityDistinctActorRule, ApplicationCapabilityElevationRule,
        ApplicationCapabilityFieldBinding, ApplicationCapabilityFieldDimension,
        ApplicationCapabilityGraphClause, ApplicationCapabilityGraphRule,
        ApplicationCapabilityMagnitudeDimension, ApplicationCapabilityPropagationComposition,
        ApplicationCapabilityRef, ApplicationCapabilityRelationBinding,
        ApplicationCapabilityRelationDimension, ApplicationCapabilitySeparationOfDutyRule,
        ApplicationCapabilityTargetDefinition, ApplicationCapabilityValidityDefinition,
        ApplicationCapabilityValidityTimeline, ApplicationCapabilityValueBinding,
        ApplicationCapabilityWorkflowDefinition,
    },
    application_schema::{
        ApplicationAuthorizationPathBuilder, ApplicationEncodedScalarValue,
        ApplicationOperationRef, ApplicationSchemaDeclarationBuilder,
        StringApplicationValueBinding,
    },
};

use crate::model::{CustomerRole, PaymentId};
use crate::schema::{
    AccountAuthorizedUser, ApprovePayment, ApprovePaymentInputBinding, AuthorizationAccount,
    AuthorizationRole, BankSchema, PaymentIdentityField, PaymentInitiator, PaymentIntent,
    PaymentSource, Principal,
};

use super::{
    ApprovedBusinessPaymentAdvance, ApprovedBusinessPaymentAdvanceOperation,
    ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentApprovalOperation,
    ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentControlContext, ApprovedBusinessPaymentInstanceStart,
    ApprovedBusinessPaymentInstanceStartOperation,
};

worth_query_decl::facade::worth_query_capability_provenance!(
    pub ApprovedBusinessPaymentGrantProvenance in BankSchema
);

pub(crate) fn install_approved_business_payment_authority(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .capability_context(ApprovedBusinessPaymentControlContext::reference())
        .capability_provenance(ApprovedBusinessPaymentGrantProvenance::reference())
        .capability(contract::<_, _>(
            ApprovedBusinessPaymentApproval::reference(),
            ApprovedBusinessPaymentApprovalOperation::reference(),
        ))
        .capability(contract::<_, _>(
            ApprovedBusinessPaymentAuthoring::reference(),
            ApprovedBusinessPaymentAuthoringOperation::reference(),
        ))
        .capability(contract::<_, _>(
            ApprovedBusinessPaymentInstanceStart::reference(),
            ApprovedBusinessPaymentInstanceStartOperation::reference(),
        ))
        .capability(contract::<_, _>(
            ApprovedBusinessPaymentAdvance::reference(),
            ApprovedBusinessPaymentAdvanceOperation::reference(),
        ))
}

fn contract<Capability, Operation>(
    capability: ApplicationCapabilityRef<BankSchema, Capability>,
    operation: ApplicationOperationRef<BankSchema, Operation, ApprovePayment>,
) -> ApplicationCapabilityContract<BankSchema, Capability, Operation, ApprovePayment>
where
    Operation: worth_query_decl::facade::application_schema::ApplicationOperationMarkerIdentity<
        BankSchema,
        InputBinding = ApprovePaymentInputBinding,
    >,
{
    ApplicationCapabilityContractBuilder::new(
        capability,
        operation,
        ApprovedBusinessPaymentGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            ApprovedBusinessPaymentGrantAction::at("ApprovedBusinessPaymentGrantAction"),
            encoded("manage-approved-business-payment-workflow"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(
            ApprovedBusinessPaymentGrantResource::reference(),
        ),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            ApprovedBusinessPaymentGrantPurpose::at("ApprovedBusinessPaymentGrantPurpose"),
            encoded("business-payment-approval"),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                ApprovedBusinessPaymentGrantStatus::at("ApprovedBusinessPaymentGrantStatus"),
                encoded("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    ApprovedBusinessPaymentGrantWorkflow::at(
                        "ApprovedBusinessPaymentGrantWorkflow",
                    ),
                )
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(PaymentIdentityField::at(
                    "PaymentIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    ApprovedBusinessPaymentGrantNotBefore::at(
                        "ApprovedBusinessPaymentGrantNotBefore",
                    ),
                )
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    ApprovedBusinessPaymentGrantNotAfter::at(
                        "ApprovedBusinessPaymentGrantNotAfter",
                    ),
                )
                .unwrap(),
            ),
        ),
        ApprovedBusinessPaymentControlContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(
            ApprovedBusinessPaymentGrantParent::reference(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            ApprovedBusinessPaymentGrantGrantor::reference(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            ApprovedBusinessPaymentGrantGrantee::reference(),
        ),
        ApplicationCapabilityFieldBinding::from_reference(
            ApprovedBusinessPaymentGrantDelegationLimit::at(
                "ApprovedBusinessPaymentGrantDelegationLimit",
            ),
        )
        .unwrap(),
        ApprovedBusinessPaymentGrantProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(AccountAuthorizedUser::reference())
        .where_equal(
            AuthorizationRole::at("AuthorizationRole"),
            crate::schema::encoded_bank_value(CustomerRole::Approver),
        )
        .expect("owned record selection is valid")
        .forward(AuthorizationAccount::reference())
        .reverse(PaymentSource::reference())
        .allow(PaymentIntent::reference());
    let deny = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(PaymentInitiator::reference())
        .deny(PaymentIntent::reference());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(allow),
            ])),
            ApplicationCapabilityDenyRule::when(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(deny),
            ])),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_decl::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    )
}

fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("approved-payment authority vocabulary is valid")
}
