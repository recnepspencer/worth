use std::sync::Arc;

use worth_runtime_world::facade::ProductBranchIncarnation;

use crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt;
use crate::domain_computation::WorthQueryProductUnpublishedApplication;

/// The only completion posture that may seal terminal owner truth.
pub struct WorthQueryPerformedInboundCompletion {
    accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
    original_incarnation: ProductBranchIncarnation,
    publication: WorthQueryProductPublicationReceipt,
}

impl WorthQueryPerformedInboundCompletion {
    pub fn publication_mut(&mut self) -> &mut WorthQueryProductPublicationReceipt {
        &mut self.publication
    }
    pub fn new(
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

    pub fn accepted(&self) -> &Arc<WorthQueryAcceptedInboundOccurrence> {
        &self.accepted
    }

    pub fn original_incarnation(&self) -> ProductBranchIncarnation {
        self.original_incarnation
    }

    pub fn publication(&self) -> &WorthQueryProductPublicationReceipt {
        &self.publication
    }
}

/// World owns any Relational effects that escaped without a product transition.
/// Retaining the accepted occurrence ties recovery to its exact source meaning.
#[must_use = "retain unpublished completion and accepted evidence for recovery"]
pub struct WorthQueryUnpublishedInboundCompletion {
    accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
    unpublished: WorthQueryProductUnpublishedApplication,
}

impl WorthQueryUnpublishedInboundCompletion {
    pub fn new(
        accepted: Arc<WorthQueryAcceptedInboundOccurrence>,
        unpublished: WorthQueryProductUnpublishedApplication,
    ) -> Self {
        Self {
            accepted,
            unpublished,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        Arc<WorthQueryAcceptedInboundOccurrence>,
        WorthQueryProductUnpublishedApplication,
    ) {
        (self.accepted, self.unpublished)
    }
}

#[derive(Debug)]
pub enum WorthQueryInboundPublicationDenial {
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    ForeignRelationalRuntime,
    ForeignProductWorld,
    OriginalPublicationCommitMismatch,
    ProductAdmission,
    CompletionPreparation(crate::domain_computation::primary_graph::provider::WorthQueryInboundCompletionPreparationDenial),
    TerminalIndexUnavailable,
    CorrelationAlreadyOwned,
}

#[must_use = "retain World completion outcome until terminal custody is settled"]
pub enum WorthQueryInboundPublicationOutcome {
    AlreadyCompleted,
    Performed(WorthQueryPerformedInboundCompletion),
    NoEffect,
    ProductUnpublished(WorthQueryUnpublishedInboundCompletion),
    Denied(WorthQueryInboundPublicationDenial),
}
