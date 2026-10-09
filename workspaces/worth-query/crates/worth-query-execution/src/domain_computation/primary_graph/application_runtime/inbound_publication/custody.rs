//! Bounded runtime custody for actual completed installed transport attempts.

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
mod maintenance;
mod publication_retry;
mod recovery;

use super::installed_transport::{
    InstalledTransportCompletion, InstalledTransportPublicationOutcome,
    PerformedInstalledTransportCompletion,
};
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup;
use crate::domain_computation::primary_graph::{
    OutstandingDispatchInFlightLease, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum InstalledTransportCustodyDenial {
    AlreadyRetained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum InstalledTransportPendingReason {
    UnknownCompletion,
    ConcurrentContinuation,
    PublicationRetryRequired,
    AllocationDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_execution::ExecutionAllocationDenialKind,
        requested_payload_bytes: Option<u64>,
    },
    StagingCardinalityOverflow,
    InputDirectoryAllocationDenied {
        requested_batches: usize,
    },
    ExecutionDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    },
    ExecutionControlStopped {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    },
    PublicationAtCapacity,
    ProductRecoveryRequired,
    RecoveryStaleProduct,
    TerminalProtectionUnavailable,
    TerminalReleaseUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum InstalledTransportResumeOutcome {
    Performed,
    Pending(InstalledTransportPendingReason),
}

enum RetainedCompletion {
    Ready(Arc<InstalledTransportCompletion>),
    /// The continuing caller holds the evidence until it returns an outcome.
    Publishing,
    PerformedPending(
        PerformedInstalledTransportCompletion,
        Option<RecoveryRelease>,
    ),
    Unpublished(
        Arc<InstalledTransportCompletion>,
        WorthQueryProductUnpublishedRecovery,
        Option<RecoveryRelease>,
    ),
    /// The continuing caller holds the evidence and recovery until it returns.
    Recovering,
    StaleRelease(Arc<InstalledTransportCompletion>, RecoveryRelease),
}

#[derive(Clone)]
enum RecoveryRelease {
    World(WorthQueryProductUnpublishedRecovery),
    Cleanup(WorthQueryProductBranchOwnerCleanup),
}

enum RecoveryProgress {
    Pending(
        WorthQueryProductUnpublishedRecovery,
        Option<RecoveryRelease>,
        InstalledTransportPendingReason,
    ),
    Performed(
        PerformedInstalledTransportCompletion,
        Option<RecoveryRelease>,
    ),
    StaleReleased,
    StalePending(RecoveryRelease),
}

struct CustodyEntry {
    state: RetainedCompletion,
    _in_flight: OutstandingDispatchInFlightLease,
    operation: Option<String>,
}

#[derive(Default)]
pub(in crate::domain_computation::primary_graph::application_runtime) struct InstalledTransportCompletionCustody
{
    entries: BTreeMap<ExternalEffectCorrelationIdentity, CustodyEntry>,
    pending_by_operation: BTreeMap<String, BTreeSet<ExternalEffectCorrelationIdentity>>,
    maintenance_cursor_by_operation: BTreeMap<String, ExternalEffectCorrelationIdentity>,
}

impl InstalledTransportCompletionCustody {
    fn retain(
        &mut self,
        evidence: InstalledTransportCompletion,
        in_flight: OutstandingDispatchInFlightLease,
    ) -> Result<(), InstalledTransportCustodyDenial> {
        let record = evidence.committed().record();
        let correlation = *record.correlation();
        if self.entries.contains_key(&correlation) {
            return Err(InstalledTransportCustodyDenial::AlreadyRetained);
        }
        let operation = record.operation_slot().map(str::to_owned);
        if let Some(operation) = &operation {
            self.pending_by_operation
                .entry(operation.clone())
                .or_default()
                .insert(correlation);
        }
        self.entries.insert(
            correlation,
            CustodyEntry {
                state: RetainedCompletion::Ready(Arc::new(evidence)),
                _in_flight: in_flight,
                operation,
            },
        );
        Ok(())
    }

    fn finish(&mut self, correlation: &ExternalEffectCorrelationIdentity) {
        let Some(entry) = self.entries.remove(correlation) else {
            return;
        };
        if let Some(operation) = entry.operation {
            if let Some(pending) = self.pending_by_operation.get_mut(&operation) {
                pending.remove(correlation);
                if pending.is_empty() {
                    self.pending_by_operation.remove(&operation);
                    self.maintenance_cursor_by_operation.remove(&operation);
                }
            }
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Retain the actual completed observation before any completion publication.
    pub(in crate::domain_computation::primary_graph) fn record_installed_transport_completion(
        &self,
        evidence: InstalledTransportCompletion,
        in_flight: OutstandingDispatchInFlightLease,
    ) -> Result<(), InstalledTransportCustodyDenial> {
        self.transport_completion_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(evidence, in_flight)
    }

    /// Dispatch admission consults this before authorizing another physical send.
    pub(in crate::domain_computation::primary_graph) fn has_retained_installed_transport_completion(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> bool {
        self.transport_completion_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entries
            .contains_key(correlation)
    }

    /// Continue one retained observation. A World-performed result stays in
    /// custody until the exact World protection and provider seal both succeed.
    pub(in crate::domain_computation::primary_graph) fn resume_installed_transport_completion(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        correlation: &ExternalEffectCorrelationIdentity,
        request: &WorthQueryRequestScope,
    ) -> InstalledTransportResumeOutcome {
        use InstalledTransportPendingReason as Pending;
        use InstalledTransportResumeOutcome as Resume;
        enum Work {
            Publish(Arc<InstalledTransportCompletion>),
            Recover(
                Arc<InstalledTransportCompletion>,
                WorthQueryProductUnpublishedRecovery,
            ),
            Seal,
        }
        let work = {
            let mut custody = self
                .transport_completion_custody
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(entry) = custody.entries.get_mut(correlation) else {
                return Resume::Pending(Pending::UnknownCompletion);
            };
            match &mut entry.state {
                RetainedCompletion::Publishing | RetainedCompletion::Recovering => {
                    return Resume::Pending(Pending::ConcurrentContinuation);
                }
                RetainedCompletion::Unpublished(evidence, recovery, prior) => {
                    if let Some(release) = prior.take() {
                        if let Err(failed) = self.finish_recovery_release(release) {
                            *prior = Some(failed);
                            return Resume::Pending(Pending::ProductRecoveryRequired);
                        }
                    }
                    let evidence = Arc::clone(evidence);
                    let recovery = recovery.clone();
                    entry.state = RetainedCompletion::Recovering;
                    Work::Recover(evidence, recovery)
                }
                RetainedCompletion::StaleRelease(evidence, release) => {
                    let evidence = Arc::clone(evidence);
                    match self.finish_recovery_release(release.clone()) {
                        Ok(()) => {
                            entry.state = RetainedCompletion::Ready(evidence);
                            self.primary_provider
                                .clear_inbound_completion_publication_pending(correlation);
                            return Resume::Pending(Pending::RecoveryStaleProduct);
                        }
                        Err(next) => {
                            entry.state = RetainedCompletion::StaleRelease(evidence, next);
                            return Resume::Pending(Pending::ProductRecoveryRequired);
                        }
                    }
                }
                RetainedCompletion::Ready(evidence) => {
                    let evidence = Arc::clone(evidence);
                    entry.state = RetainedCompletion::Publishing;
                    Work::Publish(evidence)
                }
                RetainedCompletion::PerformedPending(_, _) => Work::Seal,
            }
        };
        match work {
            Work::Publish(evidence) => {
                let outcome = self.publish_installed_transport_completion(phase, evidence, request);
                let mut custody = self
                    .transport_completion_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let entry = custody
                    .entries
                    .get_mut(correlation)
                    .expect("publishing custody slot");
                match outcome {
                    InstalledTransportPublicationOutcome::AlreadyCompleted => {
                        custody.finish(correlation);
                        return Resume::Performed;
                    }
                    InstalledTransportPublicationOutcome::Performed(performed) => {
                        entry.state = RetainedCompletion::PerformedPending(performed, None);
                    }
                    InstalledTransportPublicationOutcome::ProductUnpublished(unpublished) => {
                        let (evidence, unpublished) = unpublished.into_parts();
                        entry.state = RetainedCompletion::Unpublished(
                            evidence,
                            unpublished.into_recovery(),
                            None,
                        );
                        return Resume::Pending(Pending::ProductRecoveryRequired);
                    }
                    InstalledTransportPublicationOutcome::Denied(evidence, denial) => {
                        entry.state = RetainedCompletion::Ready(evidence);
                        return Resume::Pending(denial.pending_reason());
                    }
                    InstalledTransportPublicationOutcome::NoEffect(evidence) => {
                        entry.state = RetainedCompletion::Ready(evidence);
                        return Resume::Pending(Pending::PublicationRetryRequired);
                    }
                }
            }
            Work::Recover(evidence, recovery) => {
                let progress =
                    self.continue_transport_recovery(phase, evidence.clone(), recovery, request);
                let mut custody = self
                    .transport_completion_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let entry = custody
                    .entries
                    .get_mut(correlation)
                    .expect("recovering custody slot");
                match progress {
                    RecoveryProgress::Pending(next, prior, reason) => {
                        entry.state = RetainedCompletion::Unpublished(evidence, next, prior);
                        return Resume::Pending(reason);
                    }
                    RecoveryProgress::Performed(performed, prior) => {
                        entry.state = RetainedCompletion::PerformedPending(performed, prior);
                    }
                    RecoveryProgress::StaleReleased => {
                        entry.state = RetainedCompletion::Ready(evidence);
                        self.primary_provider
                            .clear_inbound_completion_publication_pending(correlation);
                        return Resume::Pending(Pending::RecoveryStaleProduct);
                    }
                    RecoveryProgress::StalePending(release) => {
                        entry.state = RetainedCompletion::StaleRelease(evidence, release);
                        return Resume::Pending(Pending::ProductRecoveryRequired);
                    }
                }
            }
            Work::Seal => {}
        }
        let mut custody = self
            .transport_completion_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let entry = custody
            .entries
            .get_mut(correlation)
            .expect("retained completion");
        let (performed, prior) = match &mut entry.state {
            RetainedCompletion::PerformedPending(performed, prior) => (performed, prior),
            _ => return Resume::Pending(Pending::ConcurrentContinuation),
        };
        if let Some(release) = prior.take() {
            if let Err(failed) = self.finish_recovery_release(release) {
                *prior = Some(failed);
                return Resume::Pending(Pending::ProductRecoveryRequired);
            }
        }
        if !performed.settle_fresh_delivery() {
            return Resume::Pending(Pending::TerminalReleaseUnavailable);
        }
        let identity = performed.publication().publication().commit().identity();
        let protection = match self
            .product_runtime
            .owner
            .inspection_port()
            .protect_performed_publication(identity)
        {
            Ok(protection) => protection,
            Err(_) => return Resume::Pending(Pending::TerminalProtectionUnavailable),
        };
        if self
            .primary_provider
            .release_installed_transport_dispatch_provenance(performed, protection)
            .is_err()
        {
            return Resume::Pending(Pending::TerminalReleaseUnavailable);
        }
        custody.finish(correlation);
        Resume::Performed
    }
}
