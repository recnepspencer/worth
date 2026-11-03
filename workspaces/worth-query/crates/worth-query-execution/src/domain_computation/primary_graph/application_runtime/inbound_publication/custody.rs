//! Bounded runtime custody for actual completed installed transport attempts.

use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
mod maintenance;
mod publication_retry;
mod recovery;
mod resume;

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
pub enum InstalledTransportPendingReason {
    ExecutionRequest(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    UnknownCompletion,
    ConcurrentContinuation,
    PublicationRetryRequired,
    AllocationDenied {
        stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
        kind: worth_execution::ExecutionAllocationDenialClass,
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
pub enum InstalledTransportResumeOutcome {
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
        if let Some(operation) = entry.operation
            && let Some(pending) = self.pending_by_operation.get_mut(&operation)
        {
            pending.remove(correlation);
            if pending.is_empty() {
                self.pending_by_operation.remove(&operation);
                self.maintenance_cursor_by_operation.remove(&operation);
            }
        }
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Retain the actual completed observation before any completion publication.
    pub fn record_installed_transport_completion(
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
    pub fn has_retained_installed_transport_completion(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> bool {
        self.transport_completion_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entries
            .contains_key(correlation)
    }
}
