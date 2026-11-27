//! A record's governed change log in one call: capture, open and bounded
//! paging of its record and aspect history, copied into owned steps so no
//! Native or World custody outlives the call.
//!
//! The steps are exactly the canonical facts the history walk visits: World
//! product commits and the record changes they carried, with commit, version,
//! branch, structural change and changed aspects. Actor attribution is not a
//! record-history fact; [`super::WorthQueryCausalTimeline`] pairs these steps
//! with the workflow audit trail read at the same pinned commit.
use worth_foundational::facade::AspectKey;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::{
        ApplicationHistoryQueryBinding, ApplicationQueryBinding, ApplicationQueryIntent,
        ApplicationQueryScopeBinding, ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationHistoryPage, WorthQueryApplicationProjection,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestQueryDenial,
    WorthQueryGovernedHistoryCompleteness, WorthQueryGovernedHistoryDenial,
    WorthQueryGovernedHistoryLimits, WorthQueryGovernedReadAccess,
};
use worth_relational::facade::{
    history::{BranchId, CommitId},
    identity::VersionId,
    publication::RecordStructuralChange,
    transactions::RecordRef,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

use crate::session::{WorthQueryHostSession, WorthQuerySessionSnapshot};

type SubjectBinding<S, I> = <I as ApplicationQueryIntent<S>>::Binding;
type SubjectQuery<S, I> = <SubjectBinding<S, I> as ApplicationQueryBinding<S>>::Query;
type SubjectPrincipal<S, I> =
    <SubjectBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity;
type SubjectScopeBinding<S, I> = <SubjectBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding;
type SubjectScope<S, I> = <SubjectScopeBinding<S, I> as ApplicationQueryScopeBinding<S>>::Scope;
type SubjectRow<S, I> = <<SubjectBinding<S, I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// One canonical change to a governed record, as its history recorded it.
#[derive(Clone, Debug)]
pub struct WorthQueryRecordChange {
    commit: CommitId,
    version: VersionId,
    branch: BranchId,
    record: RecordRef,
    structural_change: RecordStructuralChange,
    changed_aspects: Vec<AspectKey>,
    contains_opaque_aspect: bool,
}

impl WorthQueryRecordChange {
    pub fn commit(&self) -> CommitId {
        self.commit
    }
    pub fn version(&self) -> VersionId {
        self.version
    }
    pub fn branch(&self) -> &BranchId {
        &self.branch
    }
    pub fn record(&self) -> &RecordRef {
        &self.record
    }
    /// Created, updated or deleted, as the canonical fragment recorded it.
    pub fn structural_change(&self) -> RecordStructuralChange {
        self.structural_change
    }
    /// The selected aspects this change wrote. An equal-value update keeps an
    /// occurrence with no changed aspect.
    pub fn changed_aspects(&self) -> &[AspectKey] {
        &self.changed_aspects
    }
    pub fn contains_opaque_aspect(&self) -> bool {
        self.contains_opaque_aspect
    }
}

/// One step of a record's history walk.
#[derive(Clone, Debug)]
pub enum WorthQueryRecordHistoryStep {
    /// A World product commit the walk stepped through. `root_import` marks a
    /// genuine World root carrying its one imported canonical component.
    ProductCommit {
        commit: CompositeCommitIdentity,
        root_import: bool,
    },
    /// A canonical record change in the most recently visited product commit.
    Change(WorthQueryRecordChange),
}

/// A record's governed change log, newest first in walk order.
#[derive(Clone, Debug)]
pub struct WorthQueryRecordHistory {
    steps: Vec<WorthQueryRecordHistoryStep>,
    completeness: WorthQueryGovernedHistoryCompleteness,
}

impl WorthQueryRecordHistory {
    pub fn steps(&self) -> &[WorthQueryRecordHistoryStep] {
        &self.steps
    }
    /// Only the record changes, in walk order.
    pub fn changes(&self) -> impl Iterator<Item = &WorthQueryRecordChange> {
        self.steps.iter().filter_map(|step| match step {
            WorthQueryRecordHistoryStep::Change(change) => Some(change),
            WorthQueryRecordHistoryStep::ProductCommit { .. } => None,
        })
    }
    /// `PageLimitReached` when the walk stopped at `maximum_pages` with history left.
    pub const fn completeness(&self) -> WorthQueryGovernedHistoryCompleteness {
        self.completeness
    }
}

/// What a governed record history read needs: the SubjectRead `subject`
/// selecting the record, and the capabilities governing that read and the
/// history's metadata disclosure.
pub struct WorthQueryRecordHistoryRequest<Schema, Intent, RC, RO, RI, MC, MO, MI> {
    pub subject: Intent,
    pub read: WorthQueryGovernedReadAccess<Schema, RC, RO, RI>,
    pub metadata: WorthQueryGovernedReadAccess<Schema, MC, MO, MI>,
    pub limits: WorthQueryGovernedHistoryLimits,
}

/// A request that reads the governed history `History` declares over its
/// SubjectRead: every [`WorthQueryRecordHistoryRequest`] whose subject is
/// that SubjectRead and whose capabilities govern its scope.
pub trait WorthQueryRecordHistorySource<Schema: ApplicationSchema, History> {
    #[doc(hidden)]
    fn read_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        at: Option<&WorthQueryApplicationReadObservation>,
    ) -> Result<WorthQueryRecordHistory, WorthQueryGovernedHistoryDenial>;
}

impl<Schema, History, Intent, RC, RO, RI, MC, MO, MI> WorthQueryRecordHistorySource<Schema, History>
    for WorthQueryRecordHistoryRequest<Schema, Intent, RC, RO, RI, MC, MO, MI>
where
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema, SubjectRead = Intent::Binding>,
    Intent: ApplicationQueryIntent<Schema>,
    SubjectScopeBinding<Schema, Intent>:
        ApplicationQueryScopeResolution<Schema, SubjectPrincipal<Schema, Intent>>,
    SubjectRow<Schema, Intent>:
        WorthQueryApplicationProjection<Schema, SubjectQuery<Schema, Intent>>,
    RO: ApplicationOperationMarkerIdentity<Schema> + 'static,
    RO::InputBinding: ApplicationStructuredValueBinding<Value = RI>,
    RI: ApplicationCapabilityRequest<Schema, RC, Scope = SubjectScope<Schema, Intent>>
        + Clone
        + 'static,
    MO: ApplicationOperationMarkerIdentity<Schema> + 'static,
    MO::InputBinding: ApplicationStructuredValueBinding<Value = MI>,
    MI: ApplicationCapabilityRequest<Schema, MC, Scope = SubjectScope<Schema, Intent>>
        + Clone
        + 'static,
{
    fn read_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        at: Option<&WorthQueryApplicationReadObservation>,
    ) -> Result<WorthQueryRecordHistory, WorthQueryGovernedHistoryDenial> {
        let Self {
            subject,
            read,
            metadata,
            limits,
        } = self;
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let base = session.request_as(&principal, &scope);
        let query = match at {
            Some(observation) => base.at(observation).query(subject),
            None => base.query(subject),
        };
        let mut steps = Vec::new();
        let completeness = session
            .with_funded_advancement(&scope, |phase| {
                query.read_history_governed_in_advancement::<History, _, _, _, _, _, _>(
                    &phase,
                    read,
                    metadata,
                    limits,
                    |page| copy_page_steps(page, &mut steps),
                )
            })
            .map_err(|cause| {
                WorthQueryGovernedHistoryDenial::Request(
                    WorthQueryApplicationRequestQueryDenial::ExecutionRequest(cause),
                )
            })??;
        Ok(WorthQueryRecordHistory {
            steps,
            completeness,
        })
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Reads the governed change log of the records `request.subject`
    /// discloses, from the session branch's current head:
    /// `session.record_history::<History>(request)`.
    ///
    /// Every page re-admits the session principal for both capabilities; a
    /// revoked read or metadata grant refuses rather than returning less.
    pub fn record_history<History>(
        &self,
        request: impl WorthQueryRecordHistorySource<Schema, History>,
    ) -> Result<WorthQueryRecordHistory, WorthQueryGovernedHistoryDenial> {
        request.read_in_session(self, None)
    }
}

impl<Schema: ApplicationSchema, Program> WorthQuerySessionSnapshot<'_, '_, Schema, Program> {
    /// [`WorthQueryHostSession::record_history`] as of the pinned commit: the
    /// walk starts at [`Self::selected_commit`]. Capabilities are still
    /// admitted at the head, so a pin never widens who may read.
    pub fn record_history<History>(
        &self,
        request: impl WorthQueryRecordHistorySource<Schema, History>,
    ) -> Result<WorthQueryRecordHistory, WorthQueryGovernedHistoryDenial> {
        request.read_in_session(self.session, Some(&self.observation))
    }
}

fn copy_page_steps(
    page: &WorthQueryApplicationHistoryPage,
    steps: &mut Vec<WorthQueryRecordHistoryStep>,
) {
    if let Some(commit) = page.product_commit() {
        steps.push(WorthQueryRecordHistoryStep::ProductCommit {
            commit: commit.clone(),
            root_import: page.is_root_import(),
        });
    }
    steps.extend(page.entries().map(|entry| {
        WorthQueryRecordHistoryStep::Change(WorthQueryRecordChange {
            commit: entry.commit_id(),
            version: entry.version_id(),
            branch: entry.branch_id().clone(),
            record: entry.record().clone(),
            structural_change: entry.structural_change(),
            changed_aspects: entry.changed_aspects().cloned().collect(),
            contains_opaque_aspect: entry.contains_opaque_aspect(),
        })
    }));
}
