use super::WorthQueryDocumentHistory;
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCloseReceipt;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationProductBranchCleanupFailure,
    WorthQueryApplicationProductBranchCloseDenial, WorthQueryOwnedProductHeadCloseFailure,
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
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn close(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchCloseReceipt,
        WorthQueryDocumentHistoryCloseFailure<'runtime, S, H>,
    > {
        let Self {
            head,
            model,
            expected,
            binding,
            marker,
        } = self;
        match head.close(phase) {
            Ok(receipt) => Ok(receipt),
            Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                Err(WorthQueryDocumentHistoryCloseFailure::Retained {
                    document: Self {
                        head,
                        model,
                        expected,
                        binding,
                        marker,
                    },
                    denial,
                })
            }
            Err(WorthQueryOwnedProductHeadCloseFailure::Retired { branch, failure }) => {
                Err(WorthQueryDocumentHistoryCloseFailure::Retired { branch, failure })
            }
        }
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> std::fmt::Debug
    for WorthQueryDocumentHistoryCloseFailure<'_, S, H>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retained { denial, .. } => f.debug_tuple("Retained").field(denial).finish(),
            Self::Retired { branch, failure } => f
                .debug_struct("Retired")
                .field("branch", branch)
                .field("failure", failure)
                .finish(),
        }
    }
}
