use super::*;
use worth_query_execution::{
    facade::product::{
        WorthQueryApplicationProductBranchCleanupFailure,
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranch,
    },
    publication_boundary::document_history::WorthQueryDocumentHistoryCloseFailure,
};

#[must_use = "retain the live document or retry its actual retired cleanup owner"]
pub enum WorthQueryApplicationHistoryCloseFailure<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
> {
    Retained {
        document: WorthQueryApplicationHistoryDocument<'runtime, S, H>,
        denial: WorthQueryApplicationProductBranchCloseDenial,
    },
    Retired {
        branch: WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
    Closing(WorthQueryDocumentHistoryClosing<'runtime, S>),
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryDocument<'runtime, S, H>
{
    pub fn close(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryDocumentHistoryCloseReceipt,
        WorthQueryApplicationHistoryCloseFailure<'runtime, S, H>,
    > {
        let Self { owner, access } = self;
        match owner.close(phase) {
            Err(WorthQueryDocumentHistoryCloseFailure::Closing(closing)) => {
                Err(WorthQueryApplicationHistoryCloseFailure::Closing(closing))
            }
            Ok(receipt) => Ok(receipt),
            Err(WorthQueryDocumentHistoryCloseFailure::Retained {
                document: owner,
                denial,
            }) => Err(WorthQueryApplicationHistoryCloseFailure::Retained {
                document: Self { owner, access },
                denial,
            }),
            Err(WorthQueryDocumentHistoryCloseFailure::Retired { branch, failure }) => {
                Err(WorthQueryApplicationHistoryCloseFailure::Retired { branch, failure })
            }
        }
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> std::fmt::Debug
    for WorthQueryApplicationHistoryCloseFailure<'_, S, H>
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
