//! Owner-only continuation of accepted completion and exact World recovery.

use std::sync::{Arc, Mutex};

use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::super::{
    WorthQueryPerformedInboundCompletion, WorthQueryPrimaryGraphApplicationRuntime,
};
use super::receive::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundReceiptPosture as Posture,
};
use super::WorthQueryInboundSourcePosture;
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryAcceptedInboundOccurrence,
    WorthQueryInboundCustody, WorthQueryInboundPublicationClaim,
    WorthQueryInboundRecoveryState as State, WorthQueryInboundTerminalOwnerResult,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt;
use crate::domain_computation::{
    WorthQueryProductUnpublishedApplication, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

struct RecoveryGuard<'a> {
    custody: &'a Mutex<WorthQueryInboundCustody>,
    accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
    state: Option<State>,
}

impl RecoveryGuard<'_> {
    fn retain(&mut self, state: State) {
        self.state = Some(state);
    }

    fn finish(self, terminal: Option<Arc<WorthQueryInboundTerminalOwnerResult>>) {
        assert!(self.state.is_none());
        self.custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .finish_unpublished_recovery(&self.accepted, terminal);
    }
}

impl Drop for RecoveryGuard<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            self.custody
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .restore_unpublished_recovery(&self.accepted, state);
        }
    }
}

enum RecoveryStep {
    Retain(State, Result<Posture, Denial>),
    Terminal(Arc<WorthQueryInboundTerminalOwnerResult>),
    StaleReleased,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Revisit only evidence already admitted by this owner. Raw correlation
    /// selection stays inside the custodian; source retries enter through the
    /// installed verifier with the exact signed envelope.
    pub(in crate::domain_computation) fn progress_retained_inbound_occurrence(
        &self,
        correlation_token: [u8; 32],
        request: &WorthQueryRequestScope,
    ) -> Result<Posture, Denial> {
        let correlation = ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new(
            correlation_token,
        ));
        let accepted = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .accepted_by_correlation(&correlation)
            .ok_or(Denial::UnknownCorrelation)?;
        let claim = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .claim_retryable_publication(&accepted);
        match claim {
            WorthQueryInboundPublicationClaim::Claimed => {
                self.progress_accepted_inbound_occurrence(accepted, request)
            }
            WorthQueryInboundPublicationClaim::AtCapacity => Err(Denial::CapacityExhausted),
            WorthQueryInboundPublicationClaim::Publishing => Err(Denial::PublicationInProgress),
            WorthQueryInboundPublicationClaim::Terminal => {
                self.release_retained_inbound_terminal(&accepted)?;
                Ok(Posture::Performed)
            }
            WorthQueryInboundPublicationClaim::Unpublished => {
                self.progress_unpublished_inbound_occurrence(accepted, request)
            }
        }
    }

    pub(in crate::domain_computation) fn progress_unpublished_inbound_occurrence(
        &self,
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        request: &WorthQueryRequestScope,
    ) -> Result<Posture, Denial> {
        let state = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .claim_unpublished_recovery(&accepted)
            .map_err(|claim| match claim {
                WorthQueryInboundPublicationClaim::AtCapacity => Denial::CapacityExhausted,
                _ => Denial::PublicationInProgress,
            })?;
        let mut guard = RecoveryGuard {
            custody: &self.inbound_custody,
            accepted: Arc::clone(&accepted),
            state: Some(state),
        };
        let incarnation = accepted
            .owner()
            .committed_product_publication()
            .product_incarnation();
        let lane = self
            .primary_provider
            .application_branch_commit_lane_for_occurrence(incarnation);
        let _coordination = lane.enter();
        let state = guard
            .state
            .as_ref()
            .expect("claimed recovery is retained")
            .clone();
        match self.continue_inbound_recovery_state(&accepted, state, request) {
            RecoveryStep::Retain(state, result) => {
                guard.retain(state);
                result
            }
            RecoveryStep::Terminal(terminal) => {
                guard.state.take();
                guard.finish(Some(terminal));
                self.release_retained_inbound_terminal(&accepted)?;
                Ok(Posture::Performed)
            }
            RecoveryStep::StaleReleased => {
                guard.state.take();
                self.primary_provider
                    .clear_inbound_completion_publication_pending(
                        accepted.owner().record().correlation(),
                    );
                guard.finish(None);
                Err(Denial::RecoveryStaleProduct)
            }
        }
    }

    fn continue_inbound_recovery_state(
        &self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        state: State,
        request: &WorthQueryRequestScope,
    ) -> RecoveryStep {
        match state {
            State::Active(recovery) => {
                self.continue_active_inbound_recovery(accepted, recovery, request)
            }
            State::HandoffWorld { old, next } => self.finish_inbound_handoff(old, next),
            State::HandoffCleanup { cleanup, next } => match cleanup.retry() {
                Ok(_) => RecoveryStep::Retain(State::Active(next), Ok(Posture::AcceptedPending)),
                Err(failed) => RecoveryStep::Retain(
                    State::HandoffCleanup {
                        cleanup: failed.into_cleanup(),
                        next,
                    },
                    Err(Denial::RecoveryUnavailable),
                ),
            },
            State::PublishedWorld { old, terminal } => self.finish_inbound_published(old, terminal),
            State::PublishedCleanup { cleanup, terminal } => match cleanup.retry() {
                Ok(_) => RecoveryStep::Terminal(terminal),
                Err(failed) => RecoveryStep::Retain(
                    State::PublishedCleanup {
                        cleanup: failed.into_cleanup(),
                        terminal,
                    },
                    Err(Denial::RecoveryUnavailable),
                ),
            },
            State::StaleWorld(old) => self.finish_inbound_stale(old),
            State::StaleCleanup(cleanup) => match cleanup.retry() {
                Ok(_) => RecoveryStep::StaleReleased,
                Err(failed) => RecoveryStep::Retain(
                    State::StaleCleanup(failed.into_cleanup()),
                    Err(Denial::RecoveryUnavailable),
                ),
            },
        }
    }

    fn continue_active_inbound_recovery(
        &self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        recovery: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        request: &WorthQueryRequestScope,
    ) -> RecoveryStep {
        // One load fences new adoption. A concurrent revoke may follow this
        // admission point while this exact World attempt is already in flight.
        if self.inbound_source_posture_for_operation(accepted.operation())
            == Some(WorthQueryInboundSourcePosture::Revoked)
        {
            return RecoveryStep::Retain(State::Active(recovery), Err(Denial::SourceRevoked));
        }
        let needs_settlement = match recovery.inspect() {
            Ok(unpublished) => unpublished.relational_requires_settlement(),
            Err(_) => {
                return RecoveryStep::Retain(
                    State::Active(recovery),
                    Err(Denial::RecoveryUnavailable),
                )
            }
        };
        if needs_settlement && recovery.continue_owner_settlement().is_err() {
            return RecoveryStep::Retain(State::Active(recovery), Err(Denial::RecoveryUnavailable));
        }
        let unpublished = match recovery.inspect() {
            Ok(unpublished) => unpublished,
            Err(_) => {
                return RecoveryStep::Retain(
                    State::Active(recovery),
                    Err(Denial::RecoveryUnavailable),
                )
            }
        };
        let incarnation = accepted
            .owner()
            .committed_product_publication()
            .product_incarnation();
        let lease = match self.product_runtime.admit_product_occurrence(incarnation) {
            Ok(lease) => lease,
            Err(_) => {
                return RecoveryStep::Retain(
                    State::Active(recovery),
                    Err(Denial::RecoveryUnavailable),
                )
            }
        };
        let binding = lease.publication_binding();
        if unpublished.expected_product() != binding.observation() {
            drop(unpublished);
            drop(lease);
            drop(binding);
            return self.finish_inbound_stale(recovery);
        }
        let prepared = match binding.prepare_settled_relational_adoption(&unpublished, request) {
            Ok(prepared) => prepared,
            Err(_) => {
                return RecoveryStep::Retain(
                    State::Active(recovery),
                    Err(Denial::RecoveryUnavailable),
                )
            }
        };
        drop(unpublished);
        let receipt = WorthQueryReservedProductPublicationReceipt::new(
            binding.root_identity(),
            prepared.unpublished_recovery_handle(),
            false,
            false,
            false,
        );
        let outcome = prepared.execute();
        let world_recovery = binding.recovery();
        drop(lease);
        drop(binding);
        match outcome {
            RuntimeWorldPublicationOutcome::Performed(publication) => {
                let performed = WorthQueryPerformedInboundCompletion::new(
                    Arc::clone(accepted),
                    incarnation,
                    receipt.fill(publication.consume(), None),
                );
                let terminal = Arc::new(WorthQueryInboundTerminalOwnerResult::from_performed(
                    performed,
                ));
                self.finish_inbound_published(recovery, terminal)
            }
            RuntimeWorldPublicationOutcome::NoEffect(_) => {
                RecoveryStep::Retain(State::Active(recovery), Err(Denial::RecoveryUnavailable))
            }
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                let next = WorthQueryProductUnpublishedApplication::new(
                    effects,
                    world_recovery,
                    self.primary_provider.unpublished_idempotency_disposition(),
                )
                .into_recovery();
                self.finish_inbound_handoff(recovery, next)
            }
        }
    }

    fn finish_inbound_handoff(
        &self,
        old: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        next: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
    ) -> RecoveryStep {
        match self.release_product_publication_recovery(old, 0) {
            Ok(_) => RecoveryStep::Retain(State::Active(next), Ok(Posture::AcceptedPending)),
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed)) => {
                RecoveryStep::Retain(
                    State::HandoffWorld {
                        old: failed.into_recovery(),
                        next,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed)) => {
                RecoveryStep::Retain(
                    State::HandoffCleanup {
                        cleanup: failed.into_cleanup(),
                        next,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
        }
    }

    fn finish_inbound_published(
        &self,
        old: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
        terminal: Arc<WorthQueryInboundTerminalOwnerResult>,
    ) -> RecoveryStep {
        match self.release_product_publication_recovery(old, 0) {
            Ok(_) => RecoveryStep::Terminal(terminal),
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed)) => {
                RecoveryStep::Retain(
                    State::PublishedWorld {
                        old: failed.into_recovery(),
                        terminal,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed)) => {
                RecoveryStep::Retain(
                    State::PublishedCleanup {
                        cleanup: failed.into_cleanup(),
                        terminal,
                    },
                    Err(Denial::RecoveryUnavailable),
                )
            }
        }
    }

    fn finish_inbound_stale(
        &self,
        old: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
    ) -> RecoveryStep {
        match self.release_product_publication_recovery(old, 0) {
            Ok(_) => RecoveryStep::StaleReleased,
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::Recovery(failed)) => {
                RecoveryStep::Retain(
                    State::StaleWorld(failed.into_recovery()),
                    Err(Denial::RecoveryStaleProduct),
                )
            }
            Err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup(failed)) => {
                RecoveryStep::Retain(
                    State::StaleCleanup(failed.into_cleanup()),
                    Err(Denial::RecoveryUnavailable),
                )
            }
        }
    }
}
