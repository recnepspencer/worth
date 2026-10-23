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
        estate.action::<NotifyEstateDeathMutationBinding, super::estate_actions::DeathNotificationDelivery>().finish_action()
    } else {
        estate
    };
    let payments = ApplicationFeatureSpec::root::<BankSchema, BankPaymentsFeature>()
        .action::<ApplyOpeningFundingMutationBinding, super::money_actions::OpeningFundingDelivery>().finish_action()
        .action::<DepositMutationBinding, super::money_actions::DepositDelivery>().finish_action()
        .action::<WithdrawMutationBinding, super::money_actions::WithdrawalDelivery>().finish_action()
        .action::<SendMoneyMutationBinding, super::money_actions::SendMoneyDelivery>().finish_action()
        .action::<InitiateBusinessPaymentMutationBinding, super::payment_actions::PaymentInitiationDelivery>().finish_action()
        // These bindings prepare Query-owned workflow controls, not ordinary source effects.
        .native_action::<ApprovedBusinessPaymentAuthoringBinding>().finish_action()
        .native_action::<ApprovedBusinessPaymentApprovalBinding>().finish_action()
        .native_action::<ApprovedBusinessPaymentInstanceStartBinding>().finish_action()
        .native_action::<ApprovedBusinessPaymentAdvanceBinding>().finish_action()
        .conditional_operation::<PublishApprovedPaymentAssessment>();
    let payments = if include_ordinary_payment_approval {
        payments.action::<ApprovePaymentMutationBinding, super::workflow_actions::ApprovedPaymentEffectDelivery>().finish_action()
    } else {
        payments
    };
    vec![
        ApplicationFeatureSpec::root::<BankSchema, BankAccountsFeature>()
            .action::<ProgramDeploymentRevocationBinding, super::deployment_actions::ProgramDeploymentRevocationDelivery>().finish_action()
            .action::<CreatePersonalAccountMutationBinding, super::account_actions::PersonalAccountCreationDelivery>().finish_action()
            .action::<CreateBusinessAccountMutationBinding, super::account_actions::BusinessAccountCreationDelivery>().finish_action()
            .action::<GrantAccountAccessMutationBinding, super::account_actions::AccountAccessGrantDelivery>().finish_action()
            .action::<RevokeAccountAccessMutationBinding, super::account_actions::AccountAccessRevocationDelivery>().finish_action()
            .finish(),
        payments
            .action::<RejectPaymentMutationBinding, super::payment_actions::PaymentRejectionDelivery>().finish_action()
            .action::<ReverseJournalMutationBinding, super::money_actions::ReverseJournalDelivery>().finish_action()
            .finish(),
        estate
            .action::<FreezeEstateAccountMutationBinding, super::estate_actions::EstateAccountFreezeDelivery>().finish_action()
            .action::<RestoreFrozenEstateAccountBinding, super::estate_actions::RestoreFrozenAccountDelivery>().finish_action()
            .action::<OpenEstateCaseMutationBinding, super::estate_actions::EstateCaseOpeningDelivery>().finish_action()
            .action::<RecognizeEstateExecutorMutationBinding, super::estate_actions::ExecutorRecognitionDelivery>().finish_action()
            .action::<ReleaseEstateMutationBinding, super::estate_actions::EstateReleaseDelivery>().finish_action()
            .action::<DisburseEstateMutationBinding, super::estate_actions::EstateDisbursementDelivery>().finish_action()
            .action::<RetransmitEstateDeathNoticeMutationBinding, super::estate_actions::DeathNoticeRetransmissionDelivery>().finish_action()
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
