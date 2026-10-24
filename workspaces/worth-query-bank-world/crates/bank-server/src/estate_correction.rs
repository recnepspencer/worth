//! A recorded freeze restores account status through ordinary source-completion custody.
use crate::application_definition::RestoreFrozenAccountAction;
use crate::{
    BankApplication, BankAuthenticatedPrincipal, BankIdentityRuntime, EstateAccountFreezeAction,
};
use bank_domain::schema::{
    BankSchema, FreezeEstateAccountMutationBinding, RestoreFrozenEstateAccountBinding,
    RestoreRecordedEstateFreeze,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationCorrectionDenial, WorthQueryCorrectionHistoryLimits,
        WorthQueryOperationCorrectionPreparation,
    },
    primary_graph::{
        WorthQueryCorrectionSourceCoordinates, WorthQuerySelectedCorrectionAlternative,
    },
};

pub type BankEstateFreezeCorrectionPreparation<'a> = WorthQueryOperationCorrectionPreparation<
    'a,
    BankSchema,
    RestoreFrozenEstateAccountBinding,
    BankApplication,
    RestoreFrozenAccountAction,
>;

/// Original freeze delivery retained after fresh admission of its exact captured input.
pub type BankEstateFreezeReapplicationPreparation<'a> = WorthQueryOperationCorrectionPreparation<
    'a,
    BankSchema,
    FreezeEstateAccountMutationBinding,
    BankApplication,
    EstateAccountFreezeAction,
>;

impl BankIdentityRuntime {
    /// Reapplies a disclosed historical freeze through current original freeze capability admission.
    /// Publication, exact replay and close remain with the ordinary source-only owner.
    pub fn prepare_estate_freeze_reapplication<'a>(
        &'a self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        alternative: &WorthQuerySelectedCorrectionAlternative<'a, BankSchema>,
        key: &str,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<BankEstateFreezeReapplicationPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    {
        self.request(principal, scope).corrections().reapply(alternative).idempotency(key).limits(limits)
            .prepare_operation::<RestoreRecordedEstateFreeze, BankApplication, EstateAccountFreezeAction>(self.application_program())
    }
    /// Restores the recorded account status with fresh estate/account capability admission.
    /// The installed source-only action owns publication, replay and normal close.
    pub fn prepare_estate_freeze_correction<'a>(
        &'a self,
        principal: &BankAuthenticatedPrincipal,
        scope: &WorthQueryRequestScope,
        source: &WorthQueryCorrectionSourceCoordinates,
        key: &str,
        limits: WorthQueryCorrectionHistoryLimits,
    ) -> Result<BankEstateFreezeCorrectionPreparation<'a>, WorthQueryApplicationCorrectionDenial>
    {
        let request = self.request(principal, scope);
        let selected = request.corrections().select_coordinates(source, limits)?;
        request.corrections().reverse(&selected).idempotency(key).limits(limits)
            .prepare_operation::<RestoreRecordedEstateFreeze, BankApplication, RestoreFrozenAccountAction>(self.application_program())
    }
}
