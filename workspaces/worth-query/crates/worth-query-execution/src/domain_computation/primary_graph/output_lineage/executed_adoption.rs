//! Executed no-effect completion retains a publication, never a commit receipt.
use super::PublishedStableLineage;
use crate::domain_computation::provider_session::{
    WorthQueryMutationGraphWorkCompletion, WorthQueryProviderSessionTerminalBinding,
};

pub struct PendingExecutedOutputAdoption {
    pub(super) publication: PublishedStableLineage,
    pub(super) session: WorthQueryProviderSessionTerminalBinding,
}

/// The real executed attempt has finished its managed cleanup. Its publication
/// was adopted at the unchanged Product basis and its new reads support Ready.
pub struct WorthQueryApplicationOutputAdoption {
    publication: PublishedStableLineage,
    _completion: WorthQueryMutationGraphWorkCompletion,
}

impl PendingExecutedOutputAdoption {
    pub(in crate::domain_computation::primary_graph) fn complete(
        self,
        completion: WorthQueryMutationGraphWorkCompletion,
    ) -> Option<WorthQueryApplicationOutputAdoption> {
        let actual = completion.provider_session_binding()?;
        if !self.session.same_session(actual)
            || completion.relational_branch()
                != self
                    .publication
                    .observation()
                    .basis()
                    .relational_basis()
                    .identity()
                    .branch_id()
        {
            return None;
        }
        Some(WorthQueryApplicationOutputAdoption {
            publication: self.publication,
            _completion: completion,
        })
    }
}

impl WorthQueryApplicationOutputAdoption {
    pub(in crate::domain_computation::primary_graph) fn into_publication(
        self,
    ) -> PublishedStableLineage {
        self.publication
    }
}

impl std::fmt::Debug for PendingExecutedOutputAdoption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingExecutedOutputAdoption")
            .field("session", &self.session)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for WorthQueryApplicationOutputAdoption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationOutputAdoption")
            .finish_non_exhaustive()
    }
}
