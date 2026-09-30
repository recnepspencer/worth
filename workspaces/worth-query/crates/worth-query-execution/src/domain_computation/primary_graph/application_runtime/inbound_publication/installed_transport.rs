//! Exact installed-transport evidence for the shared World completion lane.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation;
use crate::domain_computation::primary_graph::WorthQueryInstalledTransportCompletionBinding;
use crate::domain_computation::WorthQueryProductUnpublishedApplication;
use worth_runtime_world::facade::ProductBranchIncarnation;

/// A completed physical attempt paired with its exact committed outbox owner.
/// The transport observation carries no inbound signature or verifier claim.
pub(in crate::domain_computation::primary_graph) struct InstalledTransportCompletion {
    committed: WorthQueryCommittedDispatchOutboxObservation,
    dispatch: WorthQueryExternalEffectDispatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum InstalledTransportCompletionDenial {
    NotCompleted,
    CorrelationMismatch,
    CausalMismatch,
    NoInstalledInboundContract,
}

impl InstalledTransportCompletion {
    /// Admit only the actual result of a runtime-admitted dispatch attempt.
    /// `WorthQueryExternalEffectDispatch` is sealed by the dispatch owner; the
    /// source row is independently sealed by the Relational observation owner.
    pub(in crate::domain_computation::primary_graph) fn from_observed_dispatch(
        committed: WorthQueryCommittedDispatchOutboxObservation,
        dispatch: &WorthQueryExternalEffectDispatch,
    ) -> Result<Self, InstalledTransportCompletionDenial> {
        use InstalledTransportCompletionDenial as Denial;
        if !dispatch.is_external_completion() {
            return Err(Denial::NotCompleted);
        }
        if committed.record().inbound().is_none() {
            return Err(Denial::NoInstalledInboundContract);
        }
        if committed.record().correlation() != dispatch.correlation() {
            return Err(Denial::CorrelationMismatch);
        }
        let ladder = dispatch.causal_ladder();
        if ladder
            .observation()
            .and_then(|observed| observed.predecessor())
            .is_none_or(|predecessor| predecessor.predecessor() != ladder.attempt().identity())
        {
            return Err(Denial::CausalMismatch);
        }
        Ok(Self {
            committed,
            dispatch: dispatch.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn committed(
        &self,
    ) -> &WorthQueryCommittedDispatchOutboxObservation {
        &self.committed
    }

    pub(in crate::domain_computation::primary_graph) fn dispatch(
        &self,
    ) -> &WorthQueryExternalEffectDispatch {
        &self.dispatch
    }
}

/// A World-performed terminal from an actual installed transport completion.
pub(in crate::domain_computation::primary_graph) struct PerformedInstalledTransportCompletion {
    evidence: Arc<InstalledTransportCompletion>,
    binding: WorthQueryInstalledTransportCompletionBinding,
    original_incarnation: ProductBranchIncarnation,
    publication: WorthQueryProductPublicationReceipt,
    delivery_settled: AtomicBool,
    delivery_handoff: Mutex<()>,
}

impl PerformedInstalledTransportCompletion {
    pub(super) fn new(
        evidence: Arc<InstalledTransportCompletion>,
        binding: WorthQueryInstalledTransportCompletionBinding,
        original_incarnation: ProductBranchIncarnation,
        publication: WorthQueryProductPublicationReceipt,
    ) -> Self {
        Self {
            evidence,
            binding,
            original_incarnation,
            publication,
            delivery_settled: AtomicBool::new(false),
            delivery_handoff: Mutex::new(()),
        }
    }

    pub(super) fn settle_after_conditional_handoff(&self, handoff: impl FnOnce()) -> bool {
        let _handoff = self
            .delivery_handoff
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if self.delivery_settled.load(Ordering::Acquire) {
            return true;
        }
        handoff();
        if !self.publication.settle_fresh_delivery_after_owner_handoff() {
            return false;
        }
        self.delivery_settled.store(true, Ordering::Release);
        true
    }

    pub(in crate::domain_computation::primary_graph) fn evidence(
        &self,
    ) -> &InstalledTransportCompletion {
        &self.evidence
    }
    pub(in crate::domain_computation::primary_graph) fn binding(
        &self,
    ) -> &WorthQueryInstalledTransportCompletionBinding {
        &self.binding
    }
    pub(in crate::domain_computation::primary_graph) fn original_incarnation(
        &self,
    ) -> ProductBranchIncarnation {
        self.original_incarnation
    }
    pub(in crate::domain_computation::primary_graph) fn publication(
        &self,
    ) -> &WorthQueryProductPublicationReceipt {
        &self.publication
    }
}

/// A completed transport attempt whose Relational owner effect escaped before
/// a World product transition. It must remain in managed recovery custody.
#[must_use = "retain unpublished installed transport completion for recovery"]
pub(in crate::domain_computation::primary_graph) struct UnpublishedInstalledTransportCompletion {
    evidence: Arc<InstalledTransportCompletion>,
    unpublished: WorthQueryProductUnpublishedApplication,
}

impl UnpublishedInstalledTransportCompletion {
    pub(super) fn new(
        evidence: Arc<InstalledTransportCompletion>,
        unpublished: WorthQueryProductUnpublishedApplication,
    ) -> Self {
        Self {
            evidence,
            unpublished,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        Arc<InstalledTransportCompletion>,
        WorthQueryProductUnpublishedApplication,
    ) {
        (self.evidence, self.unpublished)
    }
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum InstalledTransportPublicationDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    OriginalPublicationCommitMismatch,
    BindingUnavailable,
    PublicationPermit,
    TerminalIndexUnavailable,
    CorrelationAlreadyOwned,
    ProductAdmission,
    CompletionPreparation,
}

#[must_use = "retain actual dispatch evidence and any World recovery custody"]
pub(in crate::domain_computation::primary_graph) enum InstalledTransportPublicationOutcome {
    AlreadyCompleted,
    Performed(PerformedInstalledTransportCompletion),
    ProductUnpublished(UnpublishedInstalledTransportCompletion),
    NoEffect(Arc<InstalledTransportCompletion>),
    Denied(
        Arc<InstalledTransportCompletion>,
        InstalledTransportPublicationDenial,
    ),
}
