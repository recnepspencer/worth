//! Owner-controlled continuation from one retained accepted occurrence.

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{WorthQueryInboundPublicationOutcome, WorthQueryPrimaryGraphApplicationRuntime};
use super::receive::{WorthQueryInboundAdmissionDenial, WorthQueryInboundReceiptPosture};
use super::WorthQueryInboundSourcePosture;
use crate::domain_computation::application_aftermath::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundTerminalOwnerResult,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn progress_accepted_inbound_occurrence(
        &self,
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceiptPosture, WorthQueryInboundAdmissionDenial> {
        use WorthQueryInboundReceiptPosture as Posture;
        // This load is the revocation admission point; work already admitted
        // before a concurrent revoke may finish its in-flight World attempt.
        if self.inbound_source_posture_for_operation(accepted.operation())
            == Some(WorthQueryInboundSourcePosture::Revoked)
        {
            self.inbound_custody
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .mark_publication_retryable(&accepted);
            return Err(WorthQueryInboundAdmissionDenial::SourceRevoked);
        }
        let outcome = self.publish_inbound_completion(Arc::clone(&accepted), request);
        match outcome {
            WorthQueryInboundPublicationOutcome::AlreadyCompleted => {
                let completion_world = match self
                    .primary_provider
                    .lookup_completed_inbound(accepted.owner().record().correlation())
                {
                    Ok(Some(terminal))
                        if terminal.matches_effect(accepted.operation(), accepted.claims()) =>
                    {
                        terminal.completion_world_commit().clone()
                    }
                    _ => {
                        self.inbound_custody
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .mark_publication_retryable(&accepted);
                        return Err(WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable);
                    }
                };
                self.inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .release_accepted_after_external_terminal(&accepted, completion_world);
                Ok(Posture::AlreadyCompleted)
            }
            WorthQueryInboundPublicationOutcome::Performed(performed) => {
                let terminal = Arc::new(WorthQueryInboundTerminalOwnerResult::from_performed(
                    performed,
                ));
                let retained = self
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .retain_terminal(&accepted, terminal);
                assert!(
                    retained,
                    "performed completion has exactly one owner custody slot"
                );
                self.release_retained_inbound_terminal(&accepted)?;
                Ok(Posture::Performed)
            }
            WorthQueryInboundPublicationOutcome::ProductUnpublished(unpublished) => {
                let retained = self
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .retain_unpublished(&accepted, unpublished);
                assert!(
                    retained,
                    "unpublished completion has exactly one owner custody slot"
                );
                Ok(Posture::AcceptedPending)
            }
            WorthQueryInboundPublicationOutcome::Denied(
                super::super::WorthQueryInboundPublicationDenial::CorrelationAlreadyOwned,
            ) => {
                self.inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .mark_publication_retryable(&accepted);
                Err(WorthQueryInboundAdmissionDenial::CorrelationAlreadyOwned)
            }
            WorthQueryInboundPublicationOutcome::NoEffect(_)
            | WorthQueryInboundPublicationOutcome::Denied(_) => {
                self.inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .mark_publication_retryable(&accepted);
                Err(WorthQueryInboundAdmissionDenial::PublicationRetryRequired)
            }
        }
    }

    pub(in crate::domain_computation) fn release_retained_inbound_terminal(
        &self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) -> Result<(), WorthQueryInboundAdmissionDenial> {
        let terminal = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .terminal_for(accepted)
            .ok_or(WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable)?
            .0;
        if !self.settle_inbound_conditional_delivery(&terminal) {
            return Err(WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable);
        }
        let mut custody = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (terminal, pending) = custody
            .terminal_for(accepted)
            .ok_or(WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable)?;
        if pending {
            let world_protection = self
                .product_runtime
                .owner
                .inspection_port()
                .protect_performed_publication(
                    terminal.publication().publication().commit().identity(),
                )
                .map_err(|_| WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable)?;
            self.primary_provider
                .release_terminal_dispatch_provenance(&terminal, world_protection)
                .map_err(|_| WorthQueryInboundAdmissionDenial::TerminalCleanupUnavailable)?;
            custody.mark_terminal_release_complete(accepted);
        }
        Ok(())
    }
}
