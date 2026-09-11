//! Bank-owned read context for one exact product commit.

use std::num::NonZeroUsize;

use bank_domain::schema::BankSchema;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryRequestInterruption, WorthQueryRequestScope},
    primary_graph::{WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation},
    product::WorthQueryProductBranch,
};

use super::{BankApplicationPreviewSessionDenialKind, BankApplicationQueryDenial};

/// Opaque Bank read context whose data occurrence is reselected through
/// World-owned history while every use receives fresh request admission.
pub struct BankPreviewSession {
    branch: WorthQueryProductBranch,
    commit_ordinal: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BankPreviewSessionDiscardReceipt {
    discarded: bool,
}

impl BankPreviewSession {
    pub(crate) fn open(
        application: &WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
        request: &WorthQueryRequestScope,
    ) -> Result<Self, BankApplicationQueryDenial> {
        match request.interruption() {
            Some(WorthQueryRequestInterruption::Cancelled) => {
                return Err(BankApplicationQueryDenial::PreviewSession(
                    BankApplicationPreviewSessionDenialKind::Cancelled,
                ));
            }
            Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
                return Err(BankApplicationQueryDenial::PreviewSession(
                    BankApplicationPreviewSessionDenialKind::DeadlineExceeded,
                ));
            }
            None => {}
        }
        let branch = application.current_world();
        let selected = application
            .on_branch(branch)
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let commit_ordinal = selected.product().selected_commit().ordinal();
        Ok(Self {
            branch,
            commit_ordinal,
        })
    }

    pub(crate) fn select<'runtime>(
        &self,
        application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
        maximum_history_entries: NonZeroUsize,
    ) -> Result<WorthQuerySelectedProductOperation<'runtime, BankSchema>, BankApplicationQueryDenial>
    {
        let history = application
            .branches()
            .history(self.branch, maximum_history_entries)
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let entry = history
            .entries()
            .find(|entry| entry.selected_commit().ordinal() == self.commit_ordinal)
            .ok_or(BankApplicationQueryDenial::HistoricalCommitUnavailable)?;
        history
            .select(&entry)
            .map_err(BankApplicationQueryDenial::from_product_selection)
    }

    pub fn discard(self) -> Result<BankPreviewSessionDiscardReceipt, BankApplicationQueryDenial> {
        Ok(BankPreviewSessionDiscardReceipt { discarded: true })
    }
}

impl BankPreviewSessionDiscardReceipt {
    pub const fn discarded(self) -> bool {
        self.discarded
    }
}
