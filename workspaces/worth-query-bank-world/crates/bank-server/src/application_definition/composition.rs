use bank_domain::schema::{BankPostingIntegrity, BankSchema};
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionList, ApplicationActionRef, ApplicationCommitBoundary, ApplicationFeature,
    ApplicationFeatureInputLeaf, ApplicationFeatureLeaf, ApplicationFeatureList,
    ApplicationFeatureRef, ApplicationLocalRuleRef, ApplicationNoOutputGraph,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList, ValidatedApplicationProgram,
};

use super::governance_actions::BankEstateGovernanceActions;
use super::providers::{BankAccountsProvider, BankEstateProvider, BankPaymentsProvider};

pub struct BankApplication;
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

pub(crate) type BankFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<BankSchema, BankAccountsFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<BankSchema, BankPaymentsFeature>,
        ApplicationFeatureList<
            ApplicationFeatureRef<BankSchema, BankEstateFeature>,
            ApplicationFeatureLeaf,
        >,
    >,
>;

pub(crate) type BankRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<BankSchema, BankPaymentsFeature, BankPostingIntegrity>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

pub(crate) type BankActions = ApplicationActionList<
    ApplicationActionRef<
        BankSchema,
        BankAccountsFeature,
        bank_domain::schema::CreatePersonalAccountMutationBinding,
    >,
    ApplicationActionList<
        ApplicationActionRef<
            BankSchema,
            BankAccountsFeature,
            bank_domain::schema::CreateBusinessAccountMutationBinding,
        >,
        ApplicationActionList<
            ApplicationActionRef<
                BankSchema,
                BankAccountsFeature,
                bank_domain::schema::GrantAccountAccessMutationBinding,
            >,
            ApplicationActionList<
                ApplicationActionRef<
                    BankSchema,
                    BankAccountsFeature,
                    bank_domain::schema::RevokeAccountAccessMutationBinding,
                >,
                ApplicationActionList<
                    ApplicationActionRef<
                        BankSchema,
                        BankPaymentsFeature,
                        bank_domain::schema::ApplyOpeningFundingMutationBinding,
                    >,
                    ApplicationActionList<
                        ApplicationActionRef<
                            BankSchema,
                            BankPaymentsFeature,
                            bank_domain::schema::DepositMutationBinding,
                        >,
                        ApplicationActionList<
                            ApplicationActionRef<
                                BankSchema,
                                BankPaymentsFeature,
                                bank_domain::schema::WithdrawMutationBinding,
                            >,
                            ApplicationActionList<
                                ApplicationActionRef<
                                    BankSchema,
                                    BankPaymentsFeature,
                                    bank_domain::schema::SendMoneyMutationBinding,
                                >,
                                ApplicationActionList<
                                    ApplicationActionRef<
                                        BankSchema,
                                        BankPaymentsFeature,
                                        bank_domain::schema::InitiateBusinessPaymentMutationBinding,
                                    >,
                                    ApplicationActionList<
                                        ApplicationActionRef<
                                            BankSchema,
                                            BankPaymentsFeature,
                                            bank_domain::schema::ApprovePaymentMutationBinding,
                                        >,
                                        ApplicationActionList<
                                            ApplicationActionRef<
                                                BankSchema,
                                                BankPaymentsFeature,
                                                bank_domain::schema::RejectPaymentMutationBinding,
                                            >,
                                            ApplicationActionList<
                                                ApplicationActionRef<
                                                    BankSchema,
                                                    BankPaymentsFeature,
                                                    bank_domain::schema::ReverseJournalMutationBinding,
                                                >,
                                                ApplicationActionList<
                                                    ApplicationActionRef<
                                                        BankSchema,
                                                        BankEstateFeature,
                                                        bank_domain::schema::NotifyEstateDeathMutationBinding,
                                                    >,
                                                    ApplicationActionList<
                                                        ApplicationActionRef<
                                                            BankSchema,
                                                            BankEstateFeature,
                                                            bank_domain::schema::FreezeEstateAccountMutationBinding,
                                                        >,
                                                        ApplicationActionList<
                                                            ApplicationActionRef<
                                                                BankSchema,
                                                                BankEstateFeature,
                                                                bank_domain::schema::OpenEstateCaseMutationBinding,
                                                            >,
                                                            ApplicationActionList<
                                                                ApplicationActionRef<
                                                                    BankSchema,
                                                                    BankEstateFeature,
                                                                    bank_domain::schema::RecognizeEstateExecutorMutationBinding,
                                                                >,
                                                                    ApplicationActionList<
                                                                        ApplicationActionRef<
                                                                            BankSchema,
                                                                            BankEstateFeature,
                                                                            bank_domain::schema::ReleaseEstateMutationBinding,
                                                                        >,
                                                                        ApplicationActionList<
                                                                            ApplicationActionRef<
                                                                                BankSchema,
                                                                                BankEstateFeature,
                                                                                bank_domain::schema::DisburseEstateMutationBinding,
                                                                            >,
                                                                            ApplicationActionList<
                                                                                ApplicationActionRef<
                                                                                    BankSchema,
                                                                                    BankEstateFeature,
                                                                                    bank_domain::schema::RetransmitEstateDeathNoticeMutationBinding,
                                                                                >,
                                                                                BankEstateGovernanceActions,
                                                                            >,
                                                                        >,
                                                                    >,
                                                            >,
                                                        >,
                                                    >,
                                                >,
                                            >,
                                        >,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
>;

impl ApplicationProgramDefinition<BankSchema> for BankApplication {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Actions = BankActions;
    type Features = BankFeatures;
    type OutputGraph = ApplicationNoOutputGraph;
    type Rules = BankRules;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.application.v1");
}

pub(crate) fn validated_bank_application() -> Result<
    ValidatedApplicationProgram<BankSchema, BankApplication>,
    worth_query_host::facade::declaration::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<BankSchema, BankApplication>::begin().validated_program()
}
