use std::sync::Arc;

use worth_runtime_world::facade::{NoEffectCompositePublication, ProductBranchIncarnation};

use crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;
use crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionPreparationDenial;
use crate::domain_computation::WorthQueryProductUnpublishedApplication;

/// The only completion posture that may seal terminal owner truth.
pub(in crate::domain_computation) struct WorthQueryPerformedInboundCompletion {
    accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
    original_incarnation: ProductBranchIncarnation,
    publication: WorthQueryProductPublicationReceipt,
}

impl WorthQueryPerformedInboundCompletion {
    pub(in crate::domain_computation::primary_graph::application_runtime) fn new(
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        original_incarnation: ProductBranchIncarnation,
        publication: WorthQueryProductPublicationReceipt,
    ) -> Self {
        Self {
            accepted,
            original_incarnation,
            publication,
        }
    }

    pub(in crate::domain_computation) fn accepted(
        &self,
    ) -> &Arc<WorthQueryAcceptedInboundOccurrence> {
        &self.accepted
    }

    pub(in crate::domain_computation) fn original_incarnation(&self) -> ProductBranchIncarnation {
        self.original_incarnation
    }

    pub(in crate::domain_computation) fn publication(
        &self,
    ) -> &WorthQueryProductPublicationReceipt {
        &self.publication
    }
}

/// World owns any Relational effects that escaped without a product transition.
/// Retaining the accepted occurrence ties recovery to its exact source meaning.
#[must_use = "retain unpublished completion and accepted evidence for recovery"]
pub(in crate::domain_computation) struct WorthQueryUnpublishedInboundCompletion {
    accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
    unpublished: WorthQueryProductUnpublishedApplication,
}

impl WorthQueryUnpublishedInboundCompletion {
    pub(super) fn new(
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        unpublished: WorthQueryProductUnpublishedApplication,
    ) -> Self {
        Self {
            accepted,
            unpublished,
        }
    }

    pub(in crate::domain_computation) fn accepted(
        &self,
    ) -> &Arc<WorthQueryAcceptedInboundOccurrence> {
        &self.accepted
    }

    pub(in crate::domain_computation) fn unpublished(
        &self,
    ) -> &WorthQueryProductUnpublishedApplication {
        &self.unpublished
    }

    pub(in crate::domain_computation) fn into_parts(
        self,
    ) -> (
        Arc<WorthQueryAcceptedInboundOccurrence>,
        WorthQueryProductUnpublishedApplication,
    ) {
        (self.accepted, self.unpublished)
    }
}

#[derive(Debug)]
pub(in crate::domain_computation) enum WorthQueryInboundPublicationDenial {
    ForeignRelationalRuntime,
    ForeignProductWorld,
    OriginalPublicationCommitMismatch,
    BranchCoordinationCapacityExhausted,
    ProductAdmission(crate::basis::WorthQueryProductBranchAdmissionDenial),
    CompletionPreparation(WorthQueryInboundCompletionPreparationDenial),
    TerminalIndexUnavailable,
    CorrelationAlreadyOwned,
}

#[must_use = "retain World completion outcome until terminal custody is settled"]
pub(in crate::domain_computation) enum WorthQueryInboundPublicationOutcome {
    AlreadyCompleted,
    Performed(WorthQueryPerformedInboundCompletion),
    NoEffect(NoEffectCompositePublication),
    ProductUnpublished(WorthQueryUnpublishedInboundCompletion),
    Denied(WorthQueryInboundPublicationDenial),
}
