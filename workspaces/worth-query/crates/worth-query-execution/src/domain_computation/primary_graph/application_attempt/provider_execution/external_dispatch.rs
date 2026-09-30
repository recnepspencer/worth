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

use std::sync::Arc;

use worth_query_installation::facade::{ApplicationSchema, InstalledAftermathRecoveryContract};

use super::super::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::application_aftermath::{
    dispatch_external_effect, require_fresh_effect_authority, WorthQueryExternalEffectDispatch,
    WorthQueryExternalEffectTransport, WorthQueryPerformedExternalRedispatch,
    WorthQueryRecoveryEffectAuthority, WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial,
    WorthQueryRecoveryHandleDenialKind,
};
use crate::domain_computation::authorization::{
    WorthQueryAdmissionLapse, WorthQueryAdmittedApplicationOperation,
};
use crate::domain_computation::primary_graph::application_runtime::{
    WorthQueryExternalDispatchAdmissionDenial, WorthQueryTerminalEffectRefusal,
};
use crate::domain_computation::primary_graph::{
    InstalledTransportCompletion, InstalledTransportResumeOutcome,
    WorthQueryPrimaryGraphApplicationRuntime,
};

/// Why a host could not install an external-effect transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalTransportInstallationDenial {
    /// A transport is already installed; it is never replaced in flight.
    AlreadyInstalled,
}

/// Exact failure before an initial post-commit transport call could be made.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalDispatchPreparationDenial {
    OwnerReadDenied(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    AttemptAdmissionDenied,
    AlreadyCompleted,
    CompletionPublicationPending,
    TerminalIndexUnavailable,
    CanonicalDerivationDenied,
    TimeObservationDenied,
}

/// Why an admitted re-dispatch could not run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExternalRedispatchDenial {
    /// Fresh effect authority for the handle failed before transport: the
    /// handle already ended, it expired, or the authority is not its own.
    FreshAuthority(WorthQueryRecoveryHandleDenialKind),
    /// The admitted request was cancelled before transport.
    AdmissionCancelled,
    /// The admitted request reached its deadline before transport.
    AdmissionDeadlineExceeded,
    /// The admitted principal's authentication expired before transport.
    AdmissionAuthenticationExpired,
    /// The admission was minted by another runtime or installed binding.
    ForeignAdmission,
    RecoveryNotAdmitted,
    /// The live handle binding carries no co-committed outbox record.
    BindingOutboxMissing,
    /// No host transport is installed on this runtime.
    TransportNotInstalled,
    /// Relational could not establish the exact committed owner row.
    OwnerReadDenied(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    /// This runtime could not mint a runtime-affine physical attempt.
    AttemptAdmissionDenied,
    AlreadyCompleted,
    CompletionPublicationPending,
    TerminalIndexUnavailable,
    /// Canonical derivation for the dispatch event identity failed.
    CanonicalDerivationDenied,
    /// The installed runtime clock could not classify this physical attempt.
    TimeObservationDenied,
}

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

/// Each re-dispatch refusal keeps its own recovery denial kind, so a caller
/// never has to guess the cause behind one shared kind.
impl From<WorthQueryExternalRedispatchDenial> for WorthQueryRecoveryHandleDenial {
    fn from(denial: WorthQueryExternalRedispatchDenial) -> Self {
        use WorthQueryExternalRedispatchDenial as Redispatch;
        use WorthQueryRecoveryHandleDenialKind as Kind;
        WorthQueryRecoveryHandleDenial::new(match denial {
            Redispatch::FreshAuthority(kind) => kind,
            Redispatch::AdmissionCancelled => Kind::AdmissionCancelled,
            Redispatch::AdmissionDeadlineExceeded => Kind::AdmissionDeadlineExceeded,
            Redispatch::AdmissionAuthenticationExpired => Kind::AdmissionAuthenticationExpired,
            Redispatch::ForeignAdmission => Kind::ForeignRuntime,
            Redispatch::RecoveryNotAdmitted => Kind::RecoveryNotAdmitted,
            Redispatch::BindingOutboxMissing => Kind::DispatchOutboxMissing,
            Redispatch::TransportNotInstalled => Kind::TransportNotInstalled,
            Redispatch::OwnerReadDenied(read) => Kind::DispatchOwnerReadDenied(read),
            Redispatch::AttemptAdmissionDenied => Kind::AttemptAdmissionDenied,
            Redispatch::AlreadyCompleted => Kind::AlreadyCompleted,
            Redispatch::CompletionPublicationPending => Kind::CompletionPublicationPending,
            Redispatch::TerminalIndexUnavailable => Kind::TerminalIndexUnavailable,
            Redispatch::CanonicalDerivationDenied => Kind::CanonicalDerivationDenied,
            Redispatch::TimeObservationDenied => Kind::TimeObservationDenied,
        })
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
        self.external_effect_transport
            .set(transport)
            .map_err(|_| WorthQueryExternalTransportInstallationDenial::AlreadyInstalled)
    }

    /// True when this runtime can carry a declared effect out to its rail.
    pub fn has_external_effect_transport(&self) -> bool {
        self.external_effect_transport.get().is_some()
    }

    /// Re-dispatch the handle binding's co-committed outbox through the transport.
    ///
    /// Fresh effect authority is required before any transport call (R8.69). The
    /// outbox is read from the live handle binding — never from a caller-held
    /// receipt copy. Classification stays inside the crate-internal
    /// `dispatch_external_effect` step (R8.67). The returned proof is privately minted.
    pub fn redispatch_admitted_external_effect<Operation, Input, Scope>(
        &self,
        handle: &WorthQueryRecoveryHandle,
        authority: &WorthQueryRecoveryEffectAuthority,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryPerformedExternalRedispatch, WorthQueryExternalRedispatchDenial> {
        require_fresh_effect_authority(handle, authority)
            .map_err(|denial| WorthQueryExternalRedispatchDenial::FreshAuthority(denial.kind()))?;
        if matches!(
            handle.binding().installed_aftermath().recovery(),
            InstalledAftermathRecoveryContract::NotAdmitted
        ) {
            return Err(WorthQueryExternalRedispatchDenial::RecoveryNotAdmitted);
        }
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
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryExternalRedispatchDenial::ForeignAdmission);
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
            .primary_provider
            .committed_dispatch_outbox_for_binding(handle.binding())
            .map_err(WorthQueryExternalRedispatchDenial::OwnerReadDenied)?;
        let Some(transport) = self.external_effect_transport.get() else {
            return Err(WorthQueryExternalRedispatchDenial::TransportNotInstalled);
        };
        let dispatch = self
            .perform_committed_external_dispatch(
                transport.as_ref(),
                committed,
                admission.publication_request(),
            )
            .map_err(|denial| match denial {
                WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied => {
                    WorthQueryExternalRedispatchDenial::AttemptAdmissionDenied
                }
                WorthQueryExternalDispatchPreparationDenial::AlreadyCompleted => {
                    WorthQueryExternalRedispatchDenial::AlreadyCompleted
                }
                WorthQueryExternalDispatchPreparationDenial::CompletionPublicationPending => {
                    WorthQueryExternalRedispatchDenial::CompletionPublicationPending
                }
                WorthQueryExternalDispatchPreparationDenial::TerminalIndexUnavailable => {
                    WorthQueryExternalRedispatchDenial::TerminalIndexUnavailable
                }
                WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied => {
                    WorthQueryExternalRedispatchDenial::CanonicalDerivationDenied
                }
                WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied => {
                    WorthQueryExternalRedispatchDenial::TimeObservationDenied
                }
                WorthQueryExternalDispatchPreparationDenial::OwnerReadDenied(_) => {
                    unreachable!("owner read occurs before the common dispatch operation")
                }
            })?;
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
        outcome: WorthQueryApplicationCommitOutcome,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> WorthQueryApplicationCommitOutcome {
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            return outcome;
        };
        if receipt.dispatch_outbox().is_none() {
            return WorthQueryApplicationCommitOutcome::Committed(receipt);
        }
        let Some(transport) = self.external_effect_transport.get() else {
            return WorthQueryApplicationCommitOutcome::Committed(receipt);
        };
        let committed = match self.observe_committed_dispatch_outbox(&receipt) {
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
        match self.perform_committed_external_dispatch(transport.as_ref(), committed, request) {
            Ok(dispatch) => WorthQueryApplicationCommitOutcome::Committed(
                receipt.with_external_dispatch(dispatch),
            ),
            Err(denial) => WorthQueryApplicationCommitOutcome::Committed(
                receipt.with_external_dispatch_preparation_denial(denial),
            ),
        }
    }

    fn perform_committed_external_dispatch(
        &self,
        transport: &dyn WorthQueryExternalEffectTransport,
        committed: crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryExternalEffectDispatch, WorthQueryExternalDispatchPreparationDenial> {
        if committed.record().inbound().is_some()
            && self.has_retained_installed_transport_completion(committed.record().correlation())
        {
            let resumed = self
                .resume_installed_transport_completion(committed.record().correlation(), request);
            if matches!(resumed, InstalledTransportResumeOutcome::Performed) {
                let _ = self.progress_retained_inbound_occurrence(
                    *committed.record().correlation().bytes(),
                    request,
                );
            }
            return Err(match resumed {
                InstalledTransportResumeOutcome::Performed => {
                    WorthQueryExternalDispatchPreparationDenial::AlreadyCompleted
                }
                InstalledTransportResumeOutcome::Pending(_) => {
                    WorthQueryExternalDispatchPreparationDenial::CompletionPublicationPending
                }
            });
        }
        let completion_owner = committed.clone();
        let admitted = self
            .admit_external_dispatch_attempt(committed)
            .map_err(|denial| match denial {
                WorthQueryExternalDispatchAdmissionDenial::AlreadyCompleted => {
                    WorthQueryExternalDispatchPreparationDenial::AlreadyCompleted
                }
                WorthQueryExternalDispatchAdmissionDenial::CompletedTransportRetained => {
                    WorthQueryExternalDispatchPreparationDenial::CompletionPublicationPending
                }
                WorthQueryExternalDispatchAdmissionDenial::TerminalIndexUnavailable => {
                    WorthQueryExternalDispatchPreparationDenial::TerminalIndexUnavailable
                }
                _ => WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied,
            })?;
        let in_flight = self
            .primary_provider
            .begin_external_dispatch_in_flight(&completion_owner)
            .map_err(|_| WorthQueryExternalDispatchPreparationDenial::AttemptAdmissionDenied)?;
        let dispatch = dispatch_external_effect(transport, admitted).map_err(|denial| match denial {
            crate::domain_computation::application_aftermath::WorthQueryAftermathDerivationFailure::RuntimeTimeUnavailable => {
                WorthQueryExternalDispatchPreparationDenial::TimeObservationDenied
            }
            _ => WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied,
        })?;
        if completion_owner.record().inbound().is_some() && dispatch.is_external_completion() {
            let evidence =
                InstalledTransportCompletion::from_observed_dispatch(completion_owner, &dispatch)
                    .map_err(|_| {
                    WorthQueryExternalDispatchPreparationDenial::CanonicalDerivationDenied
                })?;
            let in_flight = in_flight.expect("inbound dispatch reserved an in-flight slot");
            // The original outbox reserved finite provenance before transport.
            // Keep its actual completion observation even if World needs retry.
            let _ = self.record_installed_transport_completion(evidence, in_flight);
            if matches!(
                self.resume_installed_transport_completion(dispatch.correlation(), request),
                InstalledTransportResumeOutcome::Performed,
            ) {
                // A callback may have entered signed custody while the
                // physical send was still in flight. Transport terminal truth
                // now owns the effect; release that redundant accepted slot.
                let _ = self
                    .progress_retained_inbound_occurrence(*dispatch.correlation().bytes(), request);
            }
        }
        Ok(dispatch)
    }
}

#[cfg(test)]
mod composite_dispatch_tests;
#[cfg(test)]
mod redispatch_denial_tests;
#[cfg(test)]
mod safe_retry_affinity_tests;

#[cfg(test)]
#[path = "external_dispatch/production_tests.rs"]
mod tests;
