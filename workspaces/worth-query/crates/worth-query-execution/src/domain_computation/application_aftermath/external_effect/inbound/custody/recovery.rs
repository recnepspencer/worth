//! Exact World obligations retained with one accepted inbound occurrence.

use std::sync::Arc;

use crate::domain_computation::application_aftermath::WorthQueryInboundTerminalOwnerResult;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup;
use crate::domain_computation::primary_graph::WorthQueryUnpublishedInboundCompletion;
use crate::domain_computation::WorthQueryProductUnpublishedRecovery;

use super::{
    MessageKey, PublicationState, WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundPublicationClaim,
};

#[must_use = "return exact World recovery to inbound owner custody"]
#[derive(Clone)]
pub(in crate::domain_computation) enum WorthQueryInboundRecoveryState {
    Recorded(crate::domain_computation::primary_graph::application_runtime::inbound_publication::recovered::RecordedCompletionContinuation),
    Active(WorthQueryProductUnpublishedRecovery),
    HandoffWorld {
        old: WorthQueryProductUnpublishedRecovery,
        next: WorthQueryProductUnpublishedRecovery,
    },
    HandoffCleanup {
        cleanup: WorthQueryProductBranchOwnerCleanup,
        next: WorthQueryProductUnpublishedRecovery,
    },
    PublishedWorld {
        old: WorthQueryProductUnpublishedRecovery,
        terminal: Arc<WorthQueryInboundTerminalOwnerResult>,
    },
    PublishedCleanup {
        cleanup: WorthQueryProductBranchOwnerCleanup,
        terminal: Arc<WorthQueryInboundTerminalOwnerResult>,
    },
    StaleWorld(WorthQueryProductUnpublishedRecovery),
    StaleCleanup(WorthQueryProductBranchOwnerCleanup),
}

impl WorthQueryInboundCustody {
    pub(in crate::domain_computation) fn recorded_completion_denial(
        &self,
        operation: &str,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Option<Arc<crate::domain_computation::primary_graph::WorthQueryRecordedCompletionDenial>>
    {
        let accepted = self.accepted_by_correlation(correlation)?;
        if accepted.operation() != operation {
            return None;
        }
        let entry = self
            .by_message
            .get(&MessageKey::from_claims(accepted.claims()))?;
        match entry.unpublished.as_ref()? {
            WorthQueryInboundRecoveryState::Recorded(original) => original.last_denial(),
            WorthQueryInboundRecoveryState::Active(_)
            | WorthQueryInboundRecoveryState::HandoffWorld { .. }
            | WorthQueryInboundRecoveryState::HandoffCleanup { .. }
            | WorthQueryInboundRecoveryState::PublishedWorld { .. }
            | WorthQueryInboundRecoveryState::PublishedCleanup { .. }
            | WorthQueryInboundRecoveryState::StaleWorld(_)
            | WorthQueryInboundRecoveryState::StaleCleanup(_) => None,
        }
    }

    pub(in crate::domain_computation) fn readmit_unpublished_completion(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        continuation: crate::domain_computation::primary_graph::application_runtime::inbound_publication::recovered::RecordedCompletionContinuation,
    ) -> bool {
        let Some(entry) = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
        else {
            return false;
        };
        if let Some(WorthQueryInboundRecoveryState::Recorded(existing)) = &entry.unpublished {
            return Arc::ptr_eq(&entry.accepted, accepted) && existing.same_original(&continuation);
        }
        if !Arc::ptr_eq(&entry.accepted, accepted)
            || entry.publication != PublicationState::Retryable
        {
            return false;
        }
        entry.unpublished = Some(WorthQueryInboundRecoveryState::Recorded(continuation));
        entry.publication = PublicationState::Unpublished;
        true
    }

    pub(in crate::domain_computation) fn finish_recorded_completion_recovery(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        completion: worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("accepted recovery remains owned");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
        assert_eq!(entry.publication, PublicationState::Recovering);
        assert!(entry.unpublished.is_none());
        // The recovering claim already owns this one publication charge.
        entry.publication = PublicationState::Publishing;
        self.release_accepted_after_external_terminal(accepted, completion);
    }
    pub(in crate::domain_computation) fn retain_unpublished(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        unpublished: WorthQueryUnpublishedInboundCompletion,
    ) -> bool {
        let Some(entry) = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
        else {
            return false;
        };
        if !Arc::ptr_eq(&entry.accepted, accepted)
            || entry.terminal.is_some()
            || entry.unpublished.is_some()
        {
            return false;
        }
        let (stored_accepted, unpublished) = unpublished.into_parts();
        assert!(Arc::ptr_eq(&stored_accepted, accepted));
        entry.unpublished = Some(WorthQueryInboundRecoveryState::Active(
            unpublished.into_recovery(),
        ));
        entry.publication = PublicationState::Unpublished;
        self.pending_by_operation
            .entry(accepted.operation().to_owned())
            .or_default()
            .insert(*accepted.owner().record().correlation());
        self.usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted")
            .publishing -= 1;
        true
    }

    pub(in crate::domain_computation) fn claim_unpublished_recovery(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
    ) -> Result<WorthQueryInboundRecoveryState, WorthQueryInboundPublicationClaim> {
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("accepted occurrence remains in owner custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
        if entry.publication != PublicationState::Unpublished {
            return Err(match entry.publication {
                PublicationState::Terminal => WorthQueryInboundPublicationClaim::Terminal,
                PublicationState::Publishing
                | PublicationState::Recovering
                | PublicationState::Retryable
                | PublicationState::Unpublished => WorthQueryInboundPublicationClaim::Publishing,
            });
        }
        let usage = self
            .usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted");
        let limit = accepted
            .owner()
            .record()
            .inbound()
            .expect("accepted occurrence retains inbound binding")
            .limits()
            .maximum_concurrent_publications
            .get();
        if usage.publishing >= limit {
            return Err(WorthQueryInboundPublicationClaim::AtCapacity);
        }
        usage.publishing += 1;
        entry.publication = PublicationState::Recovering;
        let unpublished = entry
            .unpublished
            .take()
            .expect("unpublished state retained");
        self.remove_pending(accepted);
        Ok(unpublished)
    }

    pub(in crate::domain_computation) fn restore_unpublished_recovery(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        state: WorthQueryInboundRecoveryState,
    ) {
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("accepted occurrence remains in owner custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
        assert_eq!(entry.publication, PublicationState::Recovering);
        assert!(entry.unpublished.replace(state).is_none());
        entry.publication = PublicationState::Unpublished;
        self.pending_by_operation
            .entry(accepted.operation().to_owned())
            .or_default()
            .insert(*accepted.owner().record().correlation());
        self.usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted")
            .publishing -= 1;
    }

    pub(in crate::domain_computation) fn finish_unpublished_recovery(
        &mut self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        terminal: Option<Arc<WorthQueryInboundTerminalOwnerResult>>,
    ) {
        let entry = self
            .by_message
            .get_mut(&MessageKey::from_claims(accepted.claims()))
            .expect("accepted occurrence remains in owner custody");
        assert!(Arc::ptr_eq(&entry.accepted, accepted));
        assert_eq!(entry.publication, PublicationState::Recovering);
        assert!(entry.unpublished.is_none());
        entry.publication = if let Some(terminal) = terminal {
            assert!(entry.terminal.replace(terminal).is_none());
            entry.terminal_release_pending = true;
            PublicationState::Terminal
        } else {
            PublicationState::Retryable
        };
        self.pending_by_operation
            .entry(accepted.operation().to_owned())
            .or_default()
            .insert(*accepted.owner().record().correlation());
        self.usage_by_operation
            .get_mut(accepted.operation())
            .expect("accepted operation counted")
            .publishing -= 1;
    }
}
