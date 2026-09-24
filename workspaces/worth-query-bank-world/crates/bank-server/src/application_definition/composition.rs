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
    bank_feature_specs_with_estate_notification(true)
}

fn bank_feature_specs_with_estate_notification(
    include_estate_notification: bool,
) -> Vec<ApplicationFeatureSpec> {
    use bank_domain::schema::*;

    let estate = ApplicationFeatureSpec::root::<BankSchema, BankEstateFeature>();
    let estate = if include_estate_notification {
        estate.mutation::<NotifyEstateDeathMutationBinding>()
    } else {
        estate
    };
    vec![
        ApplicationFeatureSpec::root::<BankSchema, BankAccountsFeature>()
            .mutation::<CreatePersonalAccountMutationBinding>()
            .mutation::<CreateBusinessAccountMutationBinding>()
            .mutation::<GrantAccountAccessMutationBinding>()
            .mutation::<RevokeAccountAccessMutationBinding>()
            .finish(),
        ApplicationFeatureSpec::root::<BankSchema, BankPaymentsFeature>()
            .mutation::<ApplyOpeningFundingMutationBinding>()
            .mutation::<DepositMutationBinding>()
            .mutation::<WithdrawMutationBinding>()
            .mutation::<SendMoneyMutationBinding>()
            .mutation::<InitiateBusinessPaymentMutationBinding>()
            .mutation::<ApprovedBusinessPaymentAuthoringBinding>()
            .mutation::<ApprovedBusinessPaymentApprovalBinding>()
            .mutation::<ApprovedBusinessPaymentInstanceStartBinding>()
            .mutation::<ApprovedBusinessPaymentAdvanceBinding>()
            .conditional_operation::<PublishApprovedPaymentAssessment>()
            .mutation::<ApprovePaymentMutationBinding>()
            .mutation::<RejectPaymentMutationBinding>()
            .mutation::<ReverseJournalMutationBinding>()
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

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bank_feature_specs_with_estate_notification(false)
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
