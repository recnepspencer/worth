//! "Why is this record in this state?": a record's governed change log and
//! its workflow instance's transition audit trail, read at one pinned commit.
//!
//! The two streams are canonical facts of different owners. Record history
//! carries commit, version and changed-aspect evidence but no actor; workflow
//! history carries the recorded advancer, executor, proposer and approver but
//! no per-transition commit. The timeline keeps both exactly as recorded at
//! the same basis and never joins them by inference.
use worth_query_declaration::facade::{
    application_program::{ApplicationWorkflowReadBinding, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_publication::facade::application_entry::{
    WorthQueryGovernedHistoryDenial, WorthQueryWorkflowObservationDenial,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

use super::{
    WorthQueryRecordHistory, WorthQueryRecordHistorySource, WorthQueryWorkflowAuditRequest,
    WorthQueryWorkflowAuditTrail,
};
use crate::session::{
    WorthQueryHostSession, WorthQuerySessionSnapshot, WorthQuerySessionSnapshotDenial,
};

/// A record's change log and its workflow audit trail at one commit.
#[derive(Debug)]
pub struct WorthQueryCausalTimeline {
    commit: CompositeCommitIdentity,
    record: WorthQueryRecordHistory,
    workflow: WorthQueryWorkflowAuditTrail,
}

impl WorthQueryCausalTimeline {
    /// The commit both streams were read at.
    pub fn commit(&self) -> &CompositeCommitIdentity {
        &self.commit
    }
    /// Record and aspect changes, newest first, with commit and version evidence.
    pub fn record(&self) -> &WorthQueryRecordHistory {
        &self.record
    }
    /// Workflow transitions in recorded order, with disclosed actor attribution.
    pub fn workflow(&self) -> &WorthQueryWorkflowAuditTrail {
        &self.workflow
    }
}

/// Why a causal timeline could not be read. Each stream refuses with its own
/// owner's typed denial; no partial timeline is returned.
#[derive(Debug)]
pub enum WorthQueryCausalTimelineDenial {
    Snapshot(WorthQuerySessionSnapshotDenial),
    Record(WorthQueryGovernedHistoryDenial),
    Workflow(WorthQueryWorkflowObservationDenial),
}

impl WorthQueryCausalTimelineDenial {
    /// Whether the record stream stopped at a retired history gap: the
    /// record's older history is no longer kept. Every other refusal of
    /// either stream answers `false`.
    pub const fn is_retired_history_gap(&self) -> bool {
        match self {
            Self::Record(cause) => cause.is_retired_history_gap(),
            Self::Snapshot(_) | Self::Workflow(_) => false,
        }
    }
}

impl<Schema: ApplicationSchema, Program> WorthQuerySessionSnapshot<'_, '_, Schema, Program> {
    /// Reads `record` and `workflow` at [`Self::selected_commit`]:
    /// `snapshot.causal_timeline::<History, _, SubjectRead, AuditRead>(..)`.
    pub fn causal_timeline<'w, History, W, Subject, Audit>(
        &self,
        record: impl WorthQueryRecordHistorySource<Schema, History>,
        vocabulary: WorthQueryWorkflowVocabulary<'w, Schema, W>,
        workflow: WorthQueryWorkflowAuditRequest<'_, Subject::Input, Audit::Input>,
    ) -> Result<WorthQueryCausalTimeline, WorthQueryCausalTimelineDenial>
    where
        Schema: 'w,
        W: ApplicationWorkflowSpec<Schema = Schema>,
        Subject: ApplicationWorkflowReadBinding<Schema>,
        Audit: ApplicationWorkflowReadBinding<Schema>,
        Subject::Input: Clone,
        Audit::Input: Clone,
    {
        let record = self
            .record_history(record)
            .map_err(WorthQueryCausalTimelineDenial::Record)?;
        let workflow = self
            .workflow_audit::<W, Subject, Audit>(vocabulary, workflow)
            .map_err(WorthQueryCausalTimelineDenial::Workflow)?;
        Ok(WorthQueryCausalTimeline {
            commit: self.selected_commit().clone(),
            record,
            workflow,
        })
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// [`WorthQuerySessionSnapshot::causal_timeline`] at a fresh pin of the
    /// session branch's current commit.
    pub fn causal_timeline<'w, History, W, Subject, Audit>(
        &self,
        record: impl WorthQueryRecordHistorySource<Schema, History>,
        vocabulary: WorthQueryWorkflowVocabulary<'w, Schema, W>,
        workflow: WorthQueryWorkflowAuditRequest<'_, Subject::Input, Audit::Input>,
    ) -> Result<WorthQueryCausalTimeline, WorthQueryCausalTimelineDenial>
    where
        Schema: 'w,
        W: ApplicationWorkflowSpec<Schema = Schema>,
        Subject: ApplicationWorkflowReadBinding<Schema>,
        Audit: ApplicationWorkflowReadBinding<Schema>,
        Subject::Input: Clone,
        Audit::Input: Clone,
    {
        self.snapshot()
            .map_err(WorthQueryCausalTimelineDenial::Snapshot)?
            .causal_timeline::<History, W, Subject, Audit>(record, vocabulary, workflow)
    }
}
