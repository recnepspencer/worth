//! Read-only observation of actual World-performed inbound completion.

use worth_foundational::facade::CanonicalDigestId;
use worth_runtime_world::facade::{CompositeCommitIdentity, CompositePublicationAttemptIdentity};

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use super::WorthQueryInboundVerifierHandle;
use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;

/// Read-only World identities of a completed inbound effect.
pub struct WorthQueryInboundTerminalObservation {
    original_world_commit: CompositeCommitIdentity,
    completion_world_commit: CompositeCommitIdentity,
    completion_attempt: CompositePublicationAttemptIdentity,
}

impl WorthQueryInboundTerminalObservation {
    pub const fn original_world_commit(&self) -> &CompositeCommitIdentity {
        &self.original_world_commit
    }

    pub const fn completion_world_commit(&self) -> &CompositeCommitIdentity {
        &self.completion_world_commit
    }

    pub const fn completion_attempt(&self) -> &CompositePublicationAttemptIdentity {
        &self.completion_attempt
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// A selector for inspection only. The result comes from the sealed World
    /// Performed carrier or its owner-indexed canonical summary after accepted
    /// payload custody has turned over.
    pub fn observe_inbound_terminal(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        correlation_token: [u8; 32],
    ) -> Option<WorthQueryInboundTerminalObservation> {
        self.installed_inbound_verifier(handle)?;
        let correlation = ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new(
            correlation_token,
        ));
        let terminal = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .terminal_by_correlation(&correlation);
        if let Some(terminal) = terminal {
            if terminal.accepted().operation() != handle.operation() {
                return None;
            }
            return Some(WorthQueryInboundTerminalObservation {
                original_world_commit: terminal
                    .accepted()
                    .owner()
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                completion_world_commit: terminal
                    .publication()
                    .publication()
                    .commit()
                    .identity()
                    .clone(),
                completion_attempt: terminal
                    .publication()
                    .publication()
                    .attempt_identity()
                    .clone(),
            });
        }
        let completed = self
            .primary_provider
            .lookup_completed_inbound(&correlation)
            .ok()??;
        if completed.operation() != handle.operation() {
            return None;
        }
        Some(WorthQueryInboundTerminalObservation {
            original_world_commit: completed.original_world_commit().clone(),
            completion_world_commit: completed.completion_world_commit().clone(),
            completion_attempt: completed.completion_attempt().clone(),
        })
    }
}
