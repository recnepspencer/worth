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
    worth_query_aspect, worth_query_entity, worth_query_field, worth_query_relation,
};

use crate::model::{CustomerRole, PaymentId};
use crate::schema::{
    AccountAuthorizedUser, ApprovePayment, ApprovePaymentInputBinding, AuthorizationAccount,
    AuthorizationRole, BankSchema, PaymentIdBinding, PaymentIdentityField, PaymentInitiator,
    PaymentIntent, PaymentSource, Principal,
};

use super::{
    ApprovedBusinessPaymentAdvance, ApprovedBusinessPaymentAdvanceOperation,
    ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentApprovalOperation,
    ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentControlContext, ApprovedBusinessPaymentInstanceStart,
    ApprovedBusinessPaymentInstanceStartOperation,
};

worth_query_entity!(pub ApprovedBusinessPaymentGrant for BankSchema);
worth_query_aspect!(
    pub ApprovedBusinessPaymentGrantFacts for BankSchema, ApprovedBusinessPaymentGrant;
    identity = AspectIdentity(0x91611019),
    revision = AspectContractRevision(1),
);
worth_query_field!(pub ApprovedBusinessPaymentGrantAction for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub ApprovedBusinessPaymentGrantPurpose for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: String => StringApplicationValueBinding, read_only, no_equality);
worth_query_field!(pub ApprovedBusinessPaymentGrantStatus for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ApprovedBusinessPaymentGrantWorkflow for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: PaymentId => PaymentIdBinding, read_write, no_equality);
worth_query_field!(pub ApprovedBusinessPaymentGrantNotBefore for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: u64 => worth_query_decl::facade::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ApprovedBusinessPaymentGrantNotAfter for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: u64 => worth_query_decl::facade::application_schema::U64ApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub ApprovedBusinessPaymentGrantDelegationLimit for BankSchema, ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantFacts: u64 => worth_query_decl::facade::application_schema::U64ApplicationValueBinding, read_write, no_equality);

worth_query_relation!(pub ApprovedBusinessPaymentGrantResource in BankSchema, ApprovedBusinessPaymentGrant => PaymentIntent; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ApprovedBusinessPaymentGrantParent in BankSchema, ApprovedBusinessPaymentGrant => ApprovedBusinessPaymentGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ApprovedBusinessPaymentGrantGrantor in BankSchema, Principal => ApprovedBusinessPaymentGrant; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub ApprovedBusinessPaymentGrantGrantee in BankSchema, Principal => ApprovedBusinessPaymentGrant; integrity = same_context_unbounded_retain_dangling);

worth_query_decl::facade::worth_query_capability_provenance!(
    pub ApprovedBusinessPaymentGrantProvenance in BankSchema
);

pub(crate) fn install_approved_business_payment_authority(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
        .entity(ApprovedBusinessPaymentGrant::reference())
        .aspect(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantFacts::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantAction::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantPurpose::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantStatus::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantWorkflow::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantNotBefore::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantNotAfter::reference(),
        )
        .field(
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrantDelegationLimit::reference(),
        )
        .relation(
            ApprovedBusinessPaymentGrantResource::reference(),
            ApprovedBusinessPaymentGrant::reference(),
            PaymentIntent::reference(),
        )
        .relation(
            ApprovedBusinessPaymentGrantParent::reference(),
            ApprovedBusinessPaymentGrant::reference(),
            ApprovedBusinessPaymentGrant::reference(),
        )
        .relation(
            ApprovedBusinessPaymentGrantGrantor::reference(),
            Principal::reference(),
            ApprovedBusinessPaymentGrant::reference(),
        )
        .relation(
            ApprovedBusinessPaymentGrantGrantee::reference(),
            Principal::reference(),
            ApprovedBusinessPaymentGrant::reference(),
        )
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
            ApprovedBusinessPaymentGrantAction::reference(),
            encoded("manage-approved-business-payment-workflow"),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            ApprovedBusinessPaymentGrantResource::reference(),
        ),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            ApprovedBusinessPaymentGrantPurpose::reference(),
            encoded("business-payment-approval"),
        ),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                ApprovedBusinessPaymentGrantStatus::reference(),
                encoded("active"),
            ),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    ApprovedBusinessPaymentGrantWorkflow::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(PaymentIdentityField::reference()),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    ApprovedBusinessPaymentGrantNotBefore::reference(),
                ),
                ApplicationCapabilityFieldBinding::from_reference(
                    ApprovedBusinessPaymentGrantNotAfter::reference(),
                ),
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
            ApprovedBusinessPaymentGrantDelegationLimit::reference(),
        ),
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
            AuthorizationRole::reference(),
            crate::schema::encoded_bank_value(CustomerRole::Approver),
        )
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
            ApplicationCapabilityDistinctActorRule::not_applicable(),
        ),
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
