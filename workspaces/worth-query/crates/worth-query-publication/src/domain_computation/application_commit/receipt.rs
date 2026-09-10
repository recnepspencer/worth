use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;

use super::WorthQueryApplicationCommitPublicationInspection;
use crate::application_aftermath::{
    publish_application_aftermath, WorthQueryPublishedApplicationAftermath,
    WorthQueryPublishedApplicationCommitBoundaryEvidence,
};
use crate::domain_computation::WorthQueryPublishedProductCommit;

/// Publication receipt derived from one execution-owned commit terminal.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
/// use worth_query_publication::facade::domain_computation::WorthQueryApplicationCommitPublicationReceipt;
///
/// fn counterfeit(
///     terminal: WorthQueryApplicationCommitReceipt,
/// ) -> WorthQueryApplicationCommitPublicationReceipt {
///     let _ = terminal;
///     WorthQueryApplicationCommitPublicationReceipt {
///         aftermath: todo!(),
///         boundary_evidence: todo!(),
///     }
/// }
/// ```
///
/// The publication receipt does not dereference back into execution:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt;
/// use worth_query_publication::facade::domain_computation::WorthQueryApplicationCommitPublicationReceipt;
///
/// fn escape(
///     published: &WorthQueryApplicationCommitPublicationReceipt,
/// ) -> &WorthQueryApplicationCommitReceipt {
///     published
/// }
/// ```
///
/// Nor does it expose a terminal accessor:
///
/// ```compile_fail
/// use worth_query_publication::facade::domain_computation::WorthQueryApplicationCommitPublicationReceipt;
///
/// fn escape(published: &WorthQueryApplicationCommitPublicationReceipt) {
///     let _ = published.terminal();
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCommitPublicationReceipt {
    product_commit: WorthQueryPublishedProductCommit,
    aftermath: WorthQueryPublishedApplicationAftermath,
    boundary_evidence: WorthQueryPublishedApplicationCommitBoundaryEvidence,
}

impl WorthQueryApplicationCommitPublicationReceipt {
    pub(super) fn from_terminal(terminal: WorthQueryApplicationCommitReceipt) -> Self {
        let product_commit = WorthQueryPublishedProductCommit::from_execution(
            terminal.committed_product_publication(),
        );
        let aftermath = publish_application_aftermath(&terminal);
        let boundary_evidence =
            WorthQueryPublishedApplicationCommitBoundaryEvidence::from_owner(&terminal);
        Self {
            product_commit,
            aftermath,
            boundary_evidence,
        }
    }

    pub const fn inspect(&self) -> WorthQueryApplicationCommitPublicationInspection<'_> {
        WorthQueryApplicationCommitPublicationInspection::new(&self.boundary_evidence)
    }

    pub const fn aftermath(&self) -> &WorthQueryPublishedApplicationAftermath {
        &self.aftermath
    }

    pub const fn product_commit(&self) -> &WorthQueryPublishedProductCommit {
        &self.product_commit
    }

    pub const fn boundary_evidence(&self) -> &WorthQueryPublishedApplicationCommitBoundaryEvidence {
        &self.boundary_evidence
    }
}
