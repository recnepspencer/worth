//! Recorded money correction uses the installed reversal's retained delivery owner.
use crate::{
    BankApplication, BankAuthenticatedPrincipal, BankIdentityRuntime, ReverseJournalAction,
};
use bank_domain::schema::{
    BankSchema, ReverseApprovedPayment, ReverseJournalMutationBinding, ReverseSentMoney,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationCorrectionDenial, WorthQueryCorrectionHistoryLimits,
        WorthQueryOperationCorrectionPreparation,
    },
    primary_graph::WorthQueryCorrectionSourceCoordinates,
};

pub type BankSendMoneyCorrectionPreparation<'a> = WorthQueryOperationCorrectionPreparation<
    'a,
    BankSchema,
    ReverseJournalMutationBinding,
    BankApplication,
    ReverseJournalAction,
>;

pub type BankApprovedPaymentCorrectionPreparation<'a> = WorthQueryOperationCorrectionPreparation<
    'a,
    BankSchema,
    ReverseJournalMutationBinding,
    BankApplication,
    ReverseJournalAction,
>;

/// Retained original SendMoney input with the installed action's ordinary delivery owner.
pub type BankSendMoneyReapplicationPreparation<'a> = WorthQueryOperationCorrectionPreparation<
    'a,
    BankSchema,
    bank_domain::schema::SendMoneyMutationBinding,
    BankApplication,
    crate::SendMoneyAction,
>;

impl BankIdentityRuntime {
    /// Reapplies an explicitly selected historical alternative under current source admission.
    pub fn prepare_send_money_reapplication<'a>(
        &'a self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        alternative: &worth_query_host::facade::primary_graph::WorthQuerySelectedCorrectionAlternative<'a, BankSchema>,
        key: &str,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<BankSendMoneyReapplicationPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    {
        self.request(principal, scope)
            .corrections()
            .reapply(alternative)
            .idempotency(key)
            .limits(limits)
            .prepare_operation::<ReverseSentMoney, BankApplication, crate::SendMoneyAction>(
                self.application_program(),
            )
    }

    /// Compensates the original journal after native payment and workflow settlement.
    /// Current teller admission governs the retained reversal and its close custody.
    pub fn prepare_approved_payment_correction<'a>(
        &'a self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        source: &WorthQueryCorrectionSourceCoordinates,
        key: &str,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<BankApprovedPaymentCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    {
        let request = self.request(principal, scope);
        let selected = request.corrections().select_coordinates(source, limits)?;
        request
            .corrections()
            .reverse(&selected)
            .idempotency(key)
            .limits(limits)
            .prepare_operation::<ReverseApprovedPayment, BankApplication, ReverseJournalAction>(
                self.application_program(),
            )
    }

    /// Reverses the disclosed original movement through current teller admission.
    /// Publication returns the ordinary reversal owner's advance and close custody.
    pub fn prepare_send_money_correction<'a>(
        &'a self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        source: &WorthQueryCorrectionSourceCoordinates,
        key: &str,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<BankSendMoneyCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial> {
        let request = self.request(principal, scope);
        let selected = request.corrections().select_coordinates(source, limits)?;
        request
            .corrections()
            .reverse(&selected)
            .idempotency(key)
            .limits(limits)
            .prepare_operation::<ReverseSentMoney, BankApplication, ReverseJournalAction>(
                self.application_program(),
            )
    }
}
