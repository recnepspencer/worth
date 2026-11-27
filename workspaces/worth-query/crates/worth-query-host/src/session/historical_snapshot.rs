//! As-of snapshots: a session snapshot pinned at an exact earlier commit of
//! the session's branch, for regulated audit time-travel views.
//!
//! The commit is selected through the session branch's own product history
//! under the session's current proof and policy, exactly as
//! `request.at_commit(..)`, `at_recorded_commit(..)` and
//! `at_original_receipt(..)` select it. The result is an ordinary
//! [`WorthQuerySessionSnapshot`]: every query, page and workflow read through
//! it admits the principal's current disclosure and reads the pinned commit.
use std::num::NonZeroUsize;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        DurableCompositeCommitIdentity, WorthQueryApplicationCommitReceipt,
        WorthQueryRecoveredApplicationCommitReceipt,
    },
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationHistorySelectionDenial;

use super::{WorthQueryHostSession, WorthQuerySessionSnapshot};

/// Why an as-of snapshot could not be pinned.
#[derive(Debug)]
pub enum WorthQuerySessionHistoricalSnapshotDenial {
    /// The session's policy, deadline or cancellation refused the request.
    ExecutionRequest(WorthQueryAdvancementDenial),
    /// Branch admission refused, or the commit is not in the session branch's
    /// history within `maximum_history_work` (`CommitUnavailable`).
    History(WorthQueryApplicationHistorySelectionDenial),
}

/// The historical coordinate an as-of snapshot selects.
enum HistoricalCommit<'coordinate> {
    Live(&'coordinate WorthQueryApplicationCommitReceipt),
    Recorded(DurableCompositeCommitIdentity),
    Original(&'coordinate WorthQueryRecoveredApplicationCommitReceipt),
}

impl<'home, Schema: ApplicationSchema, Program> WorthQueryHostSession<'home, Schema, Program> {
    /// Pins the commit a live receipt of this session's branch committed,
    /// visiting at most `maximum_history_work` branch history entries.
    pub fn snapshot_at_commit(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        maximum_history_work: NonZeroUsize,
    ) -> Result<
        WorthQuerySessionSnapshot<'_, 'home, Schema, Program>,
        WorthQuerySessionHistoricalSnapshotDenial,
    > {
        self.snapshot_at(HistoricalCommit::Live(receipt), maximum_history_work)
    }

    /// Pins a durable commit coordinate recorded earlier (for example in an
    /// audit log), including one recorded before this Home was reopened.
    pub fn snapshot_at_recorded_commit(
        &self,
        commit: DurableCompositeCommitIdentity,
        maximum_history_work: NonZeroUsize,
    ) -> Result<
        WorthQuerySessionSnapshot<'_, 'home, Schema, Program>,
        WorthQuerySessionHistoricalSnapshotDenial,
    > {
        self.snapshot_at(HistoricalCommit::Recorded(commit), maximum_history_work)
    }

    /// Pins the commit an original recovered receipt names.
    pub fn snapshot_at_original_receipt(
        &self,
        receipt: &WorthQueryRecoveredApplicationCommitReceipt,
        maximum_history_work: NonZeroUsize,
    ) -> Result<
        WorthQuerySessionSnapshot<'_, 'home, Schema, Program>,
        WorthQuerySessionHistoricalSnapshotDenial,
    > {
        self.snapshot_at(HistoricalCommit::Original(receipt), maximum_history_work)
    }

    fn snapshot_at(
        &self,
        commit: HistoricalCommit<'_>,
        maximum_history_work: NonZeroUsize,
    ) -> Result<
        WorthQuerySessionSnapshot<'_, 'home, Schema, Program>,
        WorthQuerySessionHistoricalSnapshotDenial,
    > {
        let scope = self.fresh_scope();
        let principal = self.call_principal();
        let request = self.request_as(&principal, &scope);
        let work = Some(maximum_history_work);
        let observation = self
            .with_funded_advancement(&scope, |phase| {
                match commit {
                    HistoricalCommit::Live(receipt) => request.at_commit(&phase, receipt, work),
                    HistoricalCommit::Recorded(commit) => {
                        request.at_recorded_commit(&phase, commit, work)
                    }
                    HistoricalCommit::Original(receipt) => {
                        request.at_original_receipt(&phase, receipt, work)
                    }
                }
                .map(|retained| retained.retained_observation())
            })
            .map_err(WorthQuerySessionHistoricalSnapshotDenial::ExecutionRequest)?
            .map_err(WorthQuerySessionHistoricalSnapshotDenial::History)?;
        Ok(WorthQuerySessionSnapshot {
            session: self,
            observation,
        })
    }
}
