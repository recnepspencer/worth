use bank_domain::estate::EstateAction;
use bank_domain::schema::{
    DisburseEstateCapability, DisburseEstateOperation, NotifyDeathEstateCapability,
    NotifyDeathEstateOperation,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    compensate_recovery_handle, dispose_recovery_handle, expire_recovery_handle,
    inspect_recovery_handle, reconcile_recovery_handle, WorthQueryRecoveryEffectAuthority,
    WorthQueryRecoveryHandle, WorthQueryRecoveryInspectAuthority,
};

use super::super::{
    recovery_types::map_expiry, BankCommitRecoveryHandle, BankEstateProgressionDenial,
    BankRecoveryDenial, BankRecoveryExpiryDecision, BankRecoveryExpiryEvaluation,
    BankRecoveryInspection, BankRecoveryTransitionReceipt,
};
use crate::{BankAuthenticatedPrincipal, BankCommitReceipt, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn open_commit_recovery(
        &self,
        receipt: &BankCommitReceipt,
    ) -> Result<BankCommitRecoveryHandle, BankRecoveryDenial> {
        receipt
            .recovery_evidence()
            .mint_recovery_handle(self.application_runtime())
            .map(|query| BankCommitRecoveryHandle { query })
            .map_err(BankRecoveryDenial::from_query)
    }

    pub(super) fn admit_commit_recovery_effect(
        &self,
        handle: &WorthQueryRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryRecoveryEffectAuthority, BankEstateProgressionDenial> {
        let admission = self.admit_notification_operation(principal, action, request)?;
        self.application_runtime()
            .admit_recovery_effect_authority(handle, &admission)
            .map_err(BankEstateProgressionDenial::from_recovery)
    }

    fn admit_commit_recovery_inspect(
        &self,
        handle: &WorthQueryRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryRecoveryInspectAuthority, BankEstateProgressionDenial> {
        if matches!(action, EstateAction::DisburseEstate(_)) {
            return self.admit_disbursement_recovery_inspect(handle, principal, action, request);
        }
        let admission = self.admit_notification_operation(principal, action, request)?;
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                NotifyDeathEstateCapability::reference(),
                NotifyDeathEstateOperation::reference(),
            )
            .map_err(BankEstateProgressionDenial::from_capability_installation)?;
        let selected = self
            .select_current_product()
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let disclosure = self
            .application_runtime()
            .admit_recovery_inspection_disclosure(&access)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        self.application_runtime()
            .admit_recovery_inspect_authority(handle, &admission, &disclosure)
            .map_err(BankEstateProgressionDenial::from_recovery)
    }

    fn admit_disbursement_recovery_inspect(
        &self,
        handle: &WorthQueryRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryRecoveryInspectAuthority, BankEstateProgressionDenial> {
        let admission = self.admit_estate_disbursement(principal, action, request)?;
        let capability = self
            .application_runtime()
            .installed_schema()
            .capability(
                DisburseEstateCapability::reference(),
                DisburseEstateOperation::reference(),
            )
            .map_err(BankEstateProgressionDenial::from_capability_installation)?;
        let selected = self
            .select_current_product()
            .map_err(BankEstateProgressionDenial::from_product_selection)?;
        let access = selected
            .admit_capability_access(principal.query(), &capability, action, request)
            .map_err(BankEstateProgressionDenial::from_authorization)?;
        let disclosure = self
            .application_runtime()
            .admit_recovery_inspection_disclosure(&access)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        self.application_runtime()
            .admit_recovery_inspect_authority(handle, &admission, &disclosure)
            .map_err(BankEstateProgressionDenial::from_recovery)
    }

    pub fn dispose_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryTransitionReceipt, BankEstateProgressionDenial> {
        let operation = handle.installed_operation().to_owned();
        let authority =
            self.admit_commit_recovery_effect(handle.query(), principal, action, request)?;
        dispose_recovery_handle(handle.query, &authority)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        Ok(BankRecoveryTransitionReceipt::new(operation))
    }

    pub fn reconcile_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryTransitionReceipt, BankEstateProgressionDenial> {
        let operation = handle.installed_operation().to_owned();
        let authority =
            self.admit_commit_recovery_effect(handle.query(), principal, action, request)?;
        reconcile_recovery_handle(handle.query, &authority)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        Ok(BankRecoveryTransitionReceipt::new(operation))
    }

    pub fn compensate_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryTransitionReceipt, BankEstateProgressionDenial> {
        let operation = handle.installed_operation().to_owned();
        let authority =
            self.admit_commit_recovery_effect(handle.query(), principal, action, request)?;
        compensate_recovery_handle(handle.query, &authority)
            .map_err(BankEstateProgressionDenial::from_recovery)?;
        Ok(BankRecoveryTransitionReceipt::new(operation))
    }

    pub fn inspect_commit_recovery(
        &self,
        handle: &BankCommitRecoveryHandle,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRecoveryInspection, BankEstateProgressionDenial> {
        let authority =
            self.admit_commit_recovery_inspect(handle.query(), principal, action, request)?;
        inspect_recovery_handle(handle.query(), &authority)
            .map(BankRecoveryInspection::from_query)
            .map_err(BankEstateProgressionDenial::from_recovery)
    }

    pub fn evaluate_commit_recovery_expiry(
        &self,
        handle: &BankCommitRecoveryHandle,
    ) -> Result<BankRecoveryExpiryEvaluation, BankRecoveryDenial> {
        self.application_runtime()
            .evaluate_recovery_expiry(handle.query())
            .map(map_expiry)
            .map_err(BankRecoveryDenial::from_query)
    }

    pub fn expire_commit_recovery(
        &self,
        handle: BankCommitRecoveryHandle,
        decision: BankRecoveryExpiryDecision,
    ) -> Result<BankRecoveryTransitionReceipt, BankRecoveryDenial> {
        let operation = handle.installed_operation().to_owned();
        expire_recovery_handle(handle.query, &decision.query)
            .map_err(BankRecoveryDenial::from_query)?;
        Ok(BankRecoveryTransitionReceipt::new(operation))
    }
}
