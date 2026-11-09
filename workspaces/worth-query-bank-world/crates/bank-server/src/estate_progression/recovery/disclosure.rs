//! Current disclosure admission reuses actual Bank operation authority, without reconstructing a receipt.
use crate::{BankAuthenticatedPrincipal, BankEstateProgressionDenial, BankIdentityRuntime};
use bank_domain::estate::EstateAction;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

impl BankIdentityRuntime {
    /// Checks current operation permission only. No admission or successor owner
    /// escapes this call; physical recovery still requires the actual native owner.
    pub fn authorize_estate_recovery_disclosure(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<(), BankEstateProgressionDenial> {
        self.application_runtime().with_application_advancement(request, |phase| {
            if matches!(action, EstateAction::DisburseEstate(_)) {
                self.admit_estate_disbursement(&phase, principal, action, request).map(drop)
            } else {
                self.admit_notification_operation(&phase, principal, action, request).map(drop)
            }
        }).map_err(|cause| BankEstateProgressionDenial::ApplicationEntry(
            Box::new(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)),
        ))?
    }
}
