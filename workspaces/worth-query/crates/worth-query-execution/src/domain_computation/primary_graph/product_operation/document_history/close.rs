use super::{
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryCloseReceipt,
    WorthQueryDocumentHistoryClosing,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationProductBranchCleanupFailure,
    WorthQueryApplicationProductBranchCloseDenial,
};
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_installation::facade::ApplicationSchema;

#[must_use = "retain the live document or its actual retired cleanup obligation"]
pub enum WorthQueryDocumentHistoryCloseFailure<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
> {
    Retained {
        document: WorthQueryDocumentHistory<'runtime, S, H>,
        denial: WorthQueryApplicationProductBranchCloseDenial,
    },
    Retired {
        branch: WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
    Closing(WorthQueryDocumentHistoryClosing<'runtime, S>),
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn close(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryDocumentHistoryCloseReceipt,
        WorthQueryDocumentHistoryCloseFailure<'runtime, S, H>,
    > {
        if phase
            .execution_request_for(&self.application().runtime.product_runtime)
            .is_err()
        {
            return Err(WorthQueryDocumentHistoryCloseFailure::Retained {
                document: self,
                denial: WorthQueryApplicationProductBranchCloseDenial::ForeignPhase,
            });
        }
        let super::selection::HistorySelection {
            head,
            journal,
            physical_source,
            navigation,
            ..
        } = self.selection.take();
        drop(physical_source);
        match WorthQueryDocumentHistoryClosing::new(head, journal, navigation).retry(phase) {
            Ok(receipt) => Ok(receipt),
            Err(closing) => match closing.into_terminal_cleanup() {
                Ok((branch, failure)) => {
                    Err(WorthQueryDocumentHistoryCloseFailure::Retired { branch, failure })
                }
                Err(closing) => Err(WorthQueryDocumentHistoryCloseFailure::Closing(closing)),
            },
        }
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> std::fmt::Debug
    for WorthQueryDocumentHistoryCloseFailure<'_, S, H>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closing(closing) => f.debug_tuple("Closing").field(closing).finish(),
            Self::Retained { denial, .. } => f.debug_tuple("Retained").field(denial).finish(),
            Self::Retired { branch, failure } => f
                .debug_struct("Retired")
                .field("branch", branch)
                .field("failure", failure)
                .finish(),
        }
    }
}
