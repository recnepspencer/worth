//! Dispatching a co-committed external effect after the commit is durable.
//!
//! Dispatch runs strictly after the mutation transaction has committed, so the
//! outbox record it reads is already recoverable truth. A host that installed
//! no transport, or an operation that declared no external effect, pays nothing
//! and observes nothing here.
//!
//! Safe-retry re-dispatch (Gate 8.7) shares this module so
//! [`dispatch_external_effect`] remains the single classification site (R8.67).

#![deny(private_interfaces)]

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

mod cleanup_dispatch;
mod committed;
pub(crate) mod denial;
pub(in crate::domain_computation) mod durable_attempt;
enum OriginalDispatchObservation<'a> {
    Starting,
    Observed(&'a WorthQueryExternalEffectDispatch),
}

use std::sync::Arc;

use worth_query_installation::facade::{ApplicationSchema, InstalledAftermathRecoveryContract};

use super::super::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::application_aftermath::{
    require_fresh_effect_authority, WorthQueryExternalEffectDispatch,
    WorthQueryExternalEffectTransport, WorthQueryPerformedExternalRedispatch,
    WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandle,
};
use crate::domain_computation::authorization::{
    WorthQueryAdmissionLapse, WorthQueryAdmittedApplicationOperation,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryTerminalEffectRefusal;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
pub use denial::{
    WorthQueryExternalDispatchAttemptDenial, WorthQueryExternalDispatchPreparationDenial,
    WorthQueryExternalRedispatchDenial, WorthQueryExternalTransportInstallationDenial,
};
pub use durable_attempt::WorthQueryDurableDispatchRecordDenial;

/// Owner-sealed material for one re-dispatch that actually crossed the
/// runtime's committed-observation dispatch operation.
///
/// The fields and constructor stay private to this module. Other crate
/// siblings may pass the seal to the recovery evidence owner, but cannot mint
/// one from a free handle identity and dispatch.
pub(crate) struct WorthQueryPerformedExternalRedispatchSeal {
    handle: crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
    dispatch: WorthQueryExternalEffectDispatch,
}

struct WorthQueryExternalRedispatchMint;

impl WorthQueryExternalRedispatchMint {
    const fn witness() -> Self {
        Self
    }
}

impl WorthQueryPerformedExternalRedispatchSeal {
    fn new(
        _mint: WorthQueryExternalRedispatchMint,
        handle: crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
        dispatch: WorthQueryExternalEffectDispatch,
    ) -> Self {
        Self { handle, dispatch }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity,
        WorthQueryExternalEffectDispatch,
    ){
        (self.handle, self.dispatch)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Installs the host's external-effect transport, once, for this runtime.
    pub fn install_external_effect_transport(
        &self,
        transport: Arc<dyn WorthQueryExternalEffectTransport>,
    ) -> Result<(), WorthQueryExternalTransportInstallationDenial> {
        self.runtime
            .external_effect_transport
            .set(transport)
            .map_err(|_| WorthQueryExternalTransportInstallationDenial::AlreadyInstalled)
    }

    /// True when this runtime can carry a declared effect out to its rail.
    pub fn has_external_effect_transport(&self) -> bool {
        self.runtime.external_effect_transport.get().is_some()
    }

    /// Re-dispatch the handle binding's co-committed outbox through the transport.
    ///
    /// Fresh effect authority is required before any transport call (R8.69). The
    /// outbox is read from the live handle binding — never from a caller-held
    /// receipt copy. Classification stays inside the crate-internal
    /// `dispatch_external_effect` step (R8.67). The returned proof is privately minted.
    pub fn redispatch_admitted_external_effect<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

        handle: &WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryEffectAuthority,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryPerformedExternalRedispatch, WorthQueryExternalRedispatchDenial> {
        match self.runtime.product_runtime.owner.owner_identity().admit_incarnation(handle.home_owner()) {
            Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => return Err(WorthQueryExternalRedispatchDenial::FreshAuthority(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenialKind::StaleIncarnation)),
            Err(worth_proof::HomeIncarnationDenial::ForeignHome) | Ok(()) => {}
        }
        let request = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryExternalRedispatchDenial::CompletionExecutionRequest(cause.into())
            })?;
        let execution = &request;

        require_fresh_effect_authority(handle, authority)
            .map_err(|denial| WorthQueryExternalRedispatchDenial::FreshAuthority(denial.kind()))?;
        if matches!(
            handle.binding().installed_aftermath().recovery(),
            InstalledAftermathRecoveryContract::NotAdmitted
        ) {
            return Err(WorthQueryExternalRedispatchDenial::RecoveryNotAdmitted);
        }
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryExternalRedispatchDenial::ForeignAdmission);
        }
        admission
            .validate_installed_operation(&self.runtime.installed_operations)
            .map_err(WorthQueryExternalRedispatchDenial::from_admission_authorization)?;
        handle
            .admit_home_owner(self.runtime.product_runtime.owner.owner_identity())
            .map_err(|denial| WorthQueryExternalRedispatchDenial::FreshAuthority(denial.kind()))?;
        admission.refuse_stale_home_authority(self.runtime.product_runtime.owner.owner_identity())
            .map_err(|_| WorthQueryExternalRedispatchDenial::FreshAuthority(crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenialKind::StaleIncarnation))?;
        if let Some(lapse) = admission.current_authority_lapse() {
            return Err(match lapse {
                WorthQueryAdmissionLapse::Cancelled => {
                    WorthQueryExternalRedispatchDenial::AdmissionCancelled
                }
                WorthQueryAdmissionLapse::DeadlineExceeded => {
                    WorthQueryExternalRedispatchDenial::AdmissionDeadlineExceeded
                }
                WorthQueryAdmissionLapse::AuthenticationExpired => {
                    WorthQueryExternalRedispatchDenial::AdmissionAuthenticationExpired
                }
            });
        }
        let Some(record) = handle.binding().dispatch_outbox() else {
            return Err(WorthQueryExternalRedispatchDenial::BindingOutboxMissing);
        };
        // A completed effect may already have released its committed owner
        // row, so the terminal owner answers before that row is read.
        self.refuse_completed_external_effect(record)
            .map_err(|refusal| match refusal {
                WorthQueryTerminalEffectRefusal::AlreadyCompleted => {
                    WorthQueryExternalRedispatchDenial::AlreadyCompleted
                }
                WorthQueryTerminalEffectRefusal::TerminalIndexUnavailable => {
                    WorthQueryExternalRedispatchDenial::TerminalIndexUnavailable
                }
            })?;
        let committed = self
            .runtime
            .primary_provider
            .committed_dispatch_outbox_for_binding(execution, handle.binding())
            .map_err(WorthQueryExternalRedispatchDenial::OwnerReadDenied)?;
        let Some(transport) = self.runtime.external_effect_transport.get() else {
            return Err(WorthQueryExternalRedispatchDenial::TransportNotInstalled);
        };
        let dispatch = self
            .perform_committed_external_dispatch(
                execution,
                phase,
                transport.as_ref(),
                committed,
                admission.publication_request(),
                |_| {},
            )
            .map_err(WorthQueryExternalDispatchPreparationDenial::redispatch_denial)?;
        Ok(WorthQueryPerformedExternalRedispatch::record(
            WorthQueryPerformedExternalRedispatchSeal::new(
                WorthQueryExternalRedispatchMint::witness(),
                handle.authority_identity(),
                dispatch,
            ),
        ))
    }

    pub(super) fn dispatch_committed_external_effect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        outcome: WorthQueryApplicationCommitOutcome,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> WorthQueryApplicationCommitOutcome {
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            return outcome;
        };
        if receipt.dispatch_outbox().is_none() {
            return WorthQueryApplicationCommitOutcome::Committed(receipt);
        }
        let Some(transport) = self.runtime.external_effect_transport.get() else {
            return WorthQueryApplicationCommitOutcome::Committed(receipt);
        };
        let committed =
            match self.observe_committed_dispatch_outbox_with_request(execution, &receipt) {
                Ok(Some(committed)) => committed,
                Ok(None) => return WorthQueryApplicationCommitOutcome::Committed(receipt),
                Err(denial) => {
                    return WorthQueryApplicationCommitOutcome::Committed(
                        receipt.with_external_dispatch_preparation_denial(
                            WorthQueryExternalDispatchPreparationDenial::OwnerReadDenied(denial),
                        ),
                    );
                }
            };
        match self.perform_committed_external_dispatch(
            execution,
            phase,
            transport.as_ref(),
            committed,
            request,
            |_| {},
        ) {
            Ok(dispatch) => WorthQueryApplicationCommitOutcome::Committed(
                receipt.with_external_dispatch(dispatch),
            ),
            Err(denial) => WorthQueryApplicationCommitOutcome::Committed(
                receipt.with_external_dispatch_preparation_denial(denial),
            ),
        }
    }
}

mod court_doors_for_test;
