//! Bounded custodian continuation independent of source signature lifetime.

use std::num::NonZeroUsize;
use std::sync::atomic::Ordering;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{InstalledTransportPendingReason, InstalledTransportResumeOutcome};
use super::receive::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundReceiptPosture as Posture,
};
use super::{WorthQueryInboundVerifierHandle, WorthQueryPrimaryGraphApplicationRuntime};

/// Counts from one finite owner-maintenance batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryInboundMaintenanceReport {
    available_before: usize,
    selected: usize,
    signed_available_before: usize,
    signed_selected: usize,
    transport_available_before: usize,
    transport_selected: usize,
    performed: usize,
    pending: usize,
    advanced: usize,
    blocked: usize,
    reclaimed: u64,
    remaining: usize,
    next_expiry_unix_seconds: Option<u64>,
}

impl WorthQueryInboundMaintenanceReport {
    pub const fn available_before(&self) -> usize {
        self.available_before
    }
    pub const fn selected(&self) -> usize {
        self.selected
    }
    pub const fn signed_available_before(&self) -> usize {
        self.signed_available_before
    }
    pub const fn signed_selected(&self) -> usize {
        self.signed_selected
    }
    pub const fn transport_available_before(&self) -> usize {
        self.transport_available_before
    }
    pub const fn transport_selected(&self) -> usize {
        self.transport_selected
    }
    pub const fn performed(&self) -> usize {
        self.performed
    }
    pub const fn pending(&self) -> usize {
        self.pending
    }
    pub const fn advanced(&self) -> usize {
        self.advanced
    }
    pub const fn blocked(&self) -> usize {
        self.blocked
    }
    pub const fn reclaimed(&self) -> u64 {
        self.reclaimed
    }
    pub const fn remaining(&self) -> usize {
        self.remaining
    }
    pub const fn next_expiry_unix_seconds(&self) -> Option<u64> {
        self.next_expiry_unix_seconds
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Continue accepted evidence and release finished custody with the exact
    /// installed route. The owner selects raw correlations internally; this
    /// entry never accepts a caller-chosen selector or re-verifies expired
    /// signed bytes. Hosts invoke it on retained-work cues or explicit owner
    /// continuation, then use the reported next expiry for one cleanup timer.
    pub fn maintain_inbound_occurrences(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        maximum_work: NonZeroUsize,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundMaintenanceReport, Denial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(Denial::ForeignVerifier)?;
        let work = maximum_work.get().min(
            usize::try_from(installed.contract.limits().maximum_discovery_work.get())
                .unwrap_or(usize::MAX),
        );
        let inbound_available = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pending_work_count(handle.operation());
        let transport_available = self.pending_transport_maintenance_count(handle.operation());
        let prefer_transport = self
            .inbound_maintenance_turn
            .fetch_add(1, Ordering::Relaxed)
            & 1
            == 1;
        let preferred_inbound_work = if inbound_available == 0 {
            0
        } else if transport_available == 0 {
            work
        } else {
            work / 2 + usize::from(work % 2 == 1 && !prefer_transport)
        };
        let mut inbound_work = preferred_inbound_work.min(inbound_available);
        let mut transport_work = (work - preferred_inbound_work).min(transport_available);
        let unused = work - inbound_work - transport_work;
        let extra_transport = unused.min(transport_available - transport_work);
        transport_work += extra_transport;
        inbound_work += (unused - extra_transport).min(inbound_available - inbound_work);
        let inbound_candidates = {
            let mut custody = self
                .inbound_custody
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            custody.maintenance_candidates(handle.operation(), inbound_work)
        };
        let (transport_candidates, _) =
            self.transport_maintenance_candidates(handle.operation(), transport_work);
        let mut report = WorthQueryInboundMaintenanceReport {
            available_before: inbound_available.saturating_add(transport_available),
            selected: inbound_candidates.len() + transport_candidates.len(),
            signed_available_before: inbound_available,
            signed_selected: inbound_candidates.len(),
            transport_available_before: transport_available,
            transport_selected: transport_candidates.len(),
            ..WorthQueryInboundMaintenanceReport::default()
        };
        for correlation in inbound_candidates {
            match self.progress_retained_inbound_occurrence(*correlation.bytes(), request) {
                Ok(Posture::Performed | Posture::AlreadyCompleted) => report.performed += 1,
                Ok(Posture::AcceptedPending) => {
                    report.pending += 1;
                    report.advanced += 1;
                }
                Ok(Posture::AlreadyAccepted) => report.pending += 1,
                Err(Denial::RecoveryStaleProduct) => {
                    report.blocked += 1;
                    if self
                        .inbound_custody
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .is_retryable_correlation(&correlation)
                    {
                        report.advanced += 1;
                    }
                }
                // This report counts blocked work; it does not translate denials.
                Err(
                    Denial::ForeignVerifier
                    | Denial::Oversized
                    | Denial::Verification(_)
                    | Denial::IncompatibleMeaning
                    | Denial::Expired
                    | Denial::ValidityWindowExceeded
                    | Denial::TimeUnavailable
                    | Denial::UnknownCorrelation
                    | Denial::RetryBeforeAcceptance
                    | Denial::ForeignOwner
                    | Denial::OriginalDispatchHasNoInboundSupport
                    | Denial::UnsupportedOutbox
                    | Denial::MessageIdentityConflict
                    | Denial::CorrelationAlreadyOwned
                    | Denial::AuthenticatedPermanent(_)
                    | Denial::CapacityExhausted
                    | Denial::TerminalCleanupUnavailable
                    | Denial::PublicationInProgress
                    | Denial::PublicationRetryRequired
                    | Denial::PublicationAllocationDenied { .. }
                    | Denial::PublicationStagingCardinalityOverflow
                    | Denial::PublicationInputDirectoryAllocationDenied { .. }
                    | Denial::PublicationExecutionDenied { .. }
                    | Denial::PublicationExecutionControlStopped { .. }
                    | Denial::RecoveryUnavailable
                    | Denial::SourceRetired
                    | Denial::SourceRevoked
                    | Denial::OwnerReadDenied(_),
                ) => report.blocked += 1,
            }
        }
        for correlation in transport_candidates {
            match self.resume_installed_transport_completion(&correlation, request) {
                InstalledTransportResumeOutcome::Performed => report.performed += 1,
                InstalledTransportResumeOutcome::Pending(
                    InstalledTransportPendingReason::RecoveryStaleProduct,
                ) => {
                    report.blocked += 1;
                    report.advanced += 1;
                }
                InstalledTransportResumeOutcome::Pending(
                    InstalledTransportPendingReason::UnknownCompletion
                    | InstalledTransportPendingReason::ConcurrentContinuation
                    | InstalledTransportPendingReason::PublicationRetryRequired
                    | InstalledTransportPendingReason::PublicationAtCapacity
                    | InstalledTransportPendingReason::ProductRecoveryRequired
                    | InstalledTransportPendingReason::TerminalProtectionUnavailable
                    | InstalledTransportPendingReason::TerminalReleaseUnavailable
                    | InstalledTransportPendingReason::AllocationDenied { .. }
                    | InstalledTransportPendingReason::StagingCardinalityOverflow
                    | InstalledTransportPendingReason::InputDirectoryAllocationDenied { .. }
                    | InstalledTransportPendingReason::ExecutionDenied { .. }
                    | InstalledTransportPendingReason::ExecutionControlStopped { .. },
                ) => report.blocked += 1,
            }
        }
        match self.cleanup_completed_inbound_occurrences(
            handle,
            NonZeroUsize::new(work).expect("installed discovery limit is nonzero"),
        ) {
            Ok(cleaned) => report.reclaimed = cleaned.reclaimed(),
            Err(_) => report.blocked += 1,
        }
        let (inbound_remaining, next_expiry) = {
            let custody = self
                .inbound_custody
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            (
                custody.pending_work_count(handle.operation()),
                custody.next_terminal_expiry(handle.operation()),
            )
        };
        report.remaining = inbound_remaining
            .saturating_add(self.pending_transport_maintenance_count(handle.operation()));
        report.next_expiry_unix_seconds = next_expiry;
        Ok(report)
    }
}
