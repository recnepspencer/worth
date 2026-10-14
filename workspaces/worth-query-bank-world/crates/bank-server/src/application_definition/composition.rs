use bank_domain::schema::{BankPostingIntegrity, BankSchema};
use worth_query_host::facade::declaration::application_program::{
    ApplicationCommitBoundary, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureSpec, ApplicationLocalRuleRef, ApplicationNoOutputGraph,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramOutputs, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ValidatedApplicationProgram,
};

use super::providers::{BankAccountsProvider, BankEstateProvider, BankPaymentsProvider};

pub struct BankApplication;
#[doc(hidden)]
pub struct BankApplicationP1;
#[doc(hidden)]
pub struct BankApplicationP2;
#[doc(hidden)]
pub struct BankAccountsFeature;
#[doc(hidden)]
pub struct BankPaymentsFeature;
#[doc(hidden)]
pub struct BankEstateFeature;

impl ApplicationFeature<BankSchema> for BankAccountsFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.bank.feature.accounts.v1";
}

impl ApplicationFeature<BankSchema> for BankPaymentsFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.bank.feature.payments.v1";
}

impl ApplicationFeature<BankSchema> for BankEstateFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.bank.feature.estate.v1";
}

pub(crate) type BankRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<BankSchema, BankPaymentsFeature, BankPostingIntegrity>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

pub(crate) fn bank_feature_specs() -> Vec<ApplicationFeatureSpec> {
    bank_feature_specs_with_optional_effects(true, true)
}

fn bank_feature_specs_with_optional_effects(
    include_estate_notification: bool,
    include_ordinary_payment_approval: bool,
) -> Vec<ApplicationFeatureSpec> {
    use bank_domain::schema::*;

    let estate = ApplicationFeatureSpec::root::<BankSchema, BankEstateFeature>();
    let estate = if include_estate_notification {
        estate.mutation::<NotifyEstateDeathMutationBinding>()
    } else {
        estate
    };
    let payments = ApplicationFeatureSpec::root::<BankSchema, BankPaymentsFeature>()
        .operation_action::<ApplyOpeningFundingMutationBinding, super::money_actions::OpeningFundingDelivery>()
        .operation_action::<DepositMutationBinding, super::money_actions::DepositDelivery>()
        .operation_action::<WithdrawMutationBinding, super::money_actions::WithdrawalDelivery>()
        .operation_action::<SendMoneyMutationBinding, super::money_actions::SendMoneyDelivery>()
        .operation_action::<InitiateBusinessPaymentMutationBinding, super::payment_actions::PaymentInitiationDelivery>()
        .mutation::<ApprovedBusinessPaymentAuthoringBinding>()
        .mutation::<ApprovedBusinessPaymentApprovalBinding>()
        .mutation::<ApprovedBusinessPaymentInstanceStartBinding>()
        .mutation::<ApprovedBusinessPaymentAdvanceBinding>()
        .conditional_operation::<PublishApprovedPaymentAssessment>();
    let payments = if include_ordinary_payment_approval {
        payments.mutation::<ApprovePaymentMutationBinding>()
    } else {
        payments
    };
    vec![
        ApplicationFeatureSpec::root::<BankSchema, BankAccountsFeature>()
            .mutation::<ProgramDeploymentRevocationBinding>()
            .operation_action::<CreatePersonalAccountMutationBinding, super::account_actions::PersonalAccountCreationDelivery>()
            .operation_action::<CreateBusinessAccountMutationBinding, super::account_actions::BusinessAccountCreationDelivery>()
            .operation_action::<GrantAccountAccessMutationBinding, super::account_actions::AccountAccessGrantDelivery>()
            .operation_action::<RevokeAccountAccessMutationBinding, super::account_actions::AccountAccessRevocationDelivery>()
            .finish(),
        payments
            .operation_action::<RejectPaymentMutationBinding, super::payment_actions::PaymentRejectionDelivery>()
            .operation_action::<ReverseJournalMutationBinding, super::money_actions::ReverseJournalDelivery>()
            .finish(),
        estate
            .mutation::<FreezeEstateAccountMutationBinding>()
            .mutation::<OpenEstateCaseMutationBinding>()
            .mutation::<RecognizeEstateExecutorMutationBinding>()
            .mutation::<ReleaseEstateMutationBinding>()
            .mutation::<DisburseEstateMutationBinding>()
            .mutation::<RetransmitEstateDeathNoticeMutationBinding>()
            .operation::<RequestEstateEmergencyAccessOperation>()
            .operation::<ApproveEstateEmergencyAccessOperation>()
            .operation::<RevokeEstateEmergencyAccessOperation>()
            .operation::<CompleteEstateMandatoryReviewOperation>()
            .operation::<DelegateEstateCapabilityOperation>()
            .operation::<RevokeEstateCapabilityOperation>()
            .finish(),
    ]
}

impl ApplicationProgramDefinition<BankSchema> for BankApplicationP1 {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.application.p1.v1");

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<BankSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<bank_domain::schema::BankProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bank_feature_specs_with_optional_effects(false, true)
    }
}

impl ApplicationProgramDefinition<BankSchema> for BankApplicationP2 {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.application.p2.v1");

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<BankSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<bank_domain::schema::BankProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bank_feature_specs_with_optional_effects(false, false)
    }
}

impl ApplicationProgramDefinition<BankSchema> for BankApplication {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.application.v1");

    fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<BankSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor.capability::<bank_domain::schema::BankProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bank_feature_specs()
    }
}

pub(crate) fn validated_bank_application() -> Result<
    ValidatedApplicationProgram<BankSchema, BankApplication>,
    worth_query_host::facade::declaration::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<BankSchema, BankApplication>::begin().validated_program()
}

pub(crate) fn validated_bank_application_p1() -> Result<
    ValidatedApplicationProgram<BankSchema, BankApplicationP1>,
    worth_query_host::facade::declaration::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<BankSchema, BankApplicationP1>::begin().validated_program()
}

pub(crate) fn validated_bank_application_p2() -> Result<
    ValidatedApplicationProgram<BankSchema, BankApplicationP2>,
    worth_query_host::facade::declaration::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<BankSchema, BankApplicationP2>::begin().validated_program()
}
