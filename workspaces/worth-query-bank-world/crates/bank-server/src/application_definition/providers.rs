use bank_domain::schema::{
    ApplyOpeningFundingMutationBinding, ApprovePaymentMutationBinding, BankAccounts, BankEstate,
    BankPayments, BankPostingIntegrity, BankSchema, CreateBusinessAccountMutationBinding,
    CreatePersonalAccountMutationBinding, DepositMutationBinding, DisburseEstateMutationBinding,
    FreezeEstateAccountMutationBinding, GrantAccountAccessMutationBinding,
    InitiateBusinessPaymentMutationBinding, NotifyEstateDeathMutationBinding,
    OpenEstateCaseMutationBinding, RecognizeEstateExecutorMutationBinding,
    RejectPaymentMutationBinding, ReleaseEstateMutationBinding,
    RetransmitEstateDeathNoticeMutationBinding, ReverseJournalMutationBinding,
    RevokeAccountAccessMutationBinding, SendMoneyMutationBinding, WithdrawMutationBinding,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    declaration::application_schema::{
        ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
        ApplicationSchemaContribution, ApplicationSchemaContributionIdentity,
        ApplicationSchemaDeclarationBuilder,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};

use crate::mutation_handlers::{
    ApplyOpeningFundingHandler, ApprovePaymentHandler, CreateBusinessAccountHandler,
    CreatePersonalAccountHandler, DepositHandler, DisburseEstateHandler,
    FreezeEstateAccountHandler, GrantAccountAccessHandler, InitiateBusinessPaymentHandler,
    NotifyEstateDeathHandler, OpenEstateCaseHandler, RecognizeEstateExecutorHandler,
    RejectPaymentHandler, ReleaseEstateHandler, RetransmitEstateDeathNoticeHandler,
    ReverseJournalHandler, RevokeAccountAccessHandler, SendMoneyHandler, WithdrawHandler,
};

macro_rules! bank_provider_contribution {
    ($Provider:ident, $Contribution:ty) => {
        #[doc(hidden)]
        pub struct $Provider;

        impl ApplicationSchemaContribution<BankSchema> for $Provider {
            const IDENTITY: ApplicationSchemaContributionIdentity =
                <$Contribution as ApplicationSchemaContribution<BankSchema>>::IDENTITY;

            fn register_members(
                schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
            ) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
                <$Contribution as ApplicationSchemaContribution<BankSchema>>::register_members(
                    schema,
                )
            }
        }
    };
}

bank_provider_contribution!(BankAccountsProvider, BankAccounts);
bank_provider_contribution!(BankPaymentsProvider, BankPayments);
bank_provider_contribution!(BankEstateProvider, BankEstate);

impl WorthQueryApplicationContribution<BankSchema> for BankAccountsProvider {
    type Configuration = ();

    fn configure(
        (): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, BankSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<CreatePersonalAccountMutationBinding, _>(CreatePersonalAccountHandler)?;
        setup.handler::<CreateBusinessAccountMutationBinding, _>(CreateBusinessAccountHandler)?;
        setup.handler::<GrantAccountAccessMutationBinding, _>(GrantAccountAccessHandler)?;
        setup.handler::<RevokeAccountAccessMutationBinding, _>(RevokeAccountAccessHandler)
    }
}

impl WorthQueryApplicationContribution<BankSchema> for BankPaymentsProvider {
    type Configuration = ();

    fn configure(
        (): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, BankSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            BankPostingIntegrity::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            super::posting_integrity::resolve_rule,
        )?;
        setup.handler::<ApplyOpeningFundingMutationBinding, _>(ApplyOpeningFundingHandler)?;
        setup.handler::<DepositMutationBinding, _>(DepositHandler)?;
        setup.handler::<WithdrawMutationBinding, _>(WithdrawHandler)?;
        setup.handler::<SendMoneyMutationBinding, _>(SendMoneyHandler)?;
        setup
            .handler::<InitiateBusinessPaymentMutationBinding, _>(InitiateBusinessPaymentHandler)?;
        setup.handler::<ApprovePaymentMutationBinding, _>(ApprovePaymentHandler)?;
        setup.handler::<RejectPaymentMutationBinding, _>(RejectPaymentHandler)?;
        setup.handler::<ReverseJournalMutationBinding, _>(ReverseJournalHandler)
    }
}

impl WorthQueryApplicationContribution<BankSchema> for BankEstateProvider {
    type Configuration = ();

    fn configure(
        (): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, BankSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<NotifyEstateDeathMutationBinding, _>(NotifyEstateDeathHandler)?;
        setup.handler::<RetransmitEstateDeathNoticeMutationBinding, _>(
            RetransmitEstateDeathNoticeHandler,
        )?;
        setup.handler::<FreezeEstateAccountMutationBinding, _>(FreezeEstateAccountHandler)?;
        setup.handler::<OpenEstateCaseMutationBinding, _>(OpenEstateCaseHandler)?;
        setup
            .handler::<RecognizeEstateExecutorMutationBinding, _>(RecognizeEstateExecutorHandler)?;
        setup.handler::<ReleaseEstateMutationBinding, _>(ReleaseEstateHandler)?;
        setup.handler::<DisburseEstateMutationBinding, _>(DisburseEstateHandler)
    }
}
