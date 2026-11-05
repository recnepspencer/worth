//! Explicit retirement custody outlives all transport/custody continuation guards.
use crate::domain_computation::primary_graph::application_inbound::TransportProgressionScopedOps;
use crate::domain_computation::primary_graph::application_inbound::inbound_publication::custody::recovery::RecoveryParentOps;
use super::*;
use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;
pub(in crate::domain_computation::primary_graph) trait ResumeScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn resume_installed_transport_completion(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        correlation: &ExternalEffectCorrelationIdentity,
        request: &WorthQueryRequestScope,
    ) -> InstalledTransportResumeOutcome {
        let mut retirements = OwnerCleanupRetirements::default();
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
                .inbound()
                .completions()
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
                    if let Some(release) = prior.take()
                        && let Err(failed) =
                            self.finish_recovery_release(execution, release, &mut retirements)
                    {
                        *prior = Some(failed);
                        return Resume::Pending(Pending::ProductRecoveryRequired);
                    }
                    let evidence = Arc::clone(evidence);
                    let recovery = recovery.clone();
                    entry.state = RetainedCompletion::Recovering;
                    Work::Recover(evidence, recovery)
                }
                RetainedCompletion::StaleRelease(evidence, release) => {
                    let evidence = Arc::clone(evidence);
                    match self.finish_recovery_release(execution, release.clone(), &mut retirements)
                    {
                        Ok(()) => {
                            entry.state = RetainedCompletion::Ready(evidence);
                            self.primary_provider()
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
                let outcome = self
                    .publish_installed_transport_completion(execution, phase, evidence, request);
                let mut custody = self
                    .inbound()
                    .completions()
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
                let progress = self.continue_transport_recovery(
                    phase,
                    evidence.clone(),
                    recovery,
                    request,
                    &mut retirements,
                );
                let mut custody = self
                    .inbound()
                    .completions()
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
                        self.primary_provider()
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
            .inbound()
            .completions()
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
        if let Some(release) = prior.take()
            && let Err(failed) = self.finish_recovery_release(execution, release, &mut retirements)
        {
            *prior = Some(failed);
            return Resume::Pending(Pending::ProductRecoveryRequired);
        }
        if !performed.settle_fresh_delivery() {
            return Resume::Pending(Pending::TerminalReleaseUnavailable);
        }
        let identity = performed.publication().publication().commit().identity();
        let protection = match self
            .product_runtime()
            .owner
            .inspection_port()
            .protect_performed_publication(identity)
        {
            Ok(protection) => protection,
            Err(_) => return Resume::Pending(Pending::TerminalProtectionUnavailable),
        };
        if self
            .primary_provider()
            .release_installed_transport_dispatch_provenance(performed, protection)
            .is_err()
        {
            return Resume::Pending(Pending::TerminalReleaseUnavailable);
        }
        custody.finish(correlation);
        Resume::Performed
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> ResumeScopedOps<Schema> for HostRuntime {}

pub(crate) mod court_doors_for_test;
use crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity;
pub use court_doors_for_test::ResumeDoorsOps;
