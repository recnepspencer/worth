//! A recorded freeze restores account status through ordinary source-completion custody.
use crate::application_definition::RestoreFrozenAccountAction;
use crate::{BankApplication, BankAuthenticatedPrincipal, BankIdentityRuntime};
use bank_domain::schema::{
    BankSchema, RestoreFrozenEstateAccountBinding, RestoreRecordedEstateFreeze,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationCorrectionDenial, WorthQueryCorrectionHistoryLimits,
        WorthQueryOperationCorrectionPreparation,
    },
    primary_graph::WorthQueryCorrectionSourceCoordinates,
};

pub type BankEstateFreezeCorrectionPreparation<'a> = WorthQueryOperationCorrectionPreparation<
    'a,
    BankSchema,
    RestoreFrozenEstateAccountBinding,
    BankApplication,
    RestoreFrozenAccountAction,
>;

impl BankIdentityRuntime {
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
