//! Passive observation of a workflow Home: governed workflow reads at a
//! session snapshot's pinned basis (on the caller's thread or an owned
//! session's call worker), and the Home's commit notifications.
use super::super::observation::input as read_input;
use super::*;
use document::programs::WorkflowOwnerProgram;
use document::schema::DocumentRetentionSchema;
use std::sync::mpsc::{channel, Receiver};
use worth_query_host::facade::{
    application_installation::WorthQueryWorkflowVocabulary, WorthQueryCommitWatchGuard,
    WorthQueryCommittedBasisVersion, WorthQueryPending, WorthQuerySessionSnapshot,
};
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Bounds every wait on the notifier or a call worker; never reached normally.
const PATIENCE: Duration = Duration::from_secs(120);

/// One governed read of the original instance and its recorded history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Frontier {
    pub(super) commit: CompositeCommitIdentity,
    /// The read's standing: a command locator only when its basis is current.
    pub(super) standing: Standing,
    pub(super) state: WorthQueryWorkflowObservedState,
    pub(super) paths: Vec<String>,
    pub(super) history: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Standing {
    Live(PublishedWorkflowInstanceRef),
    Historical(WorthQueryWorkflowHistoricalPosture),
}

impl Frontier {
    /// The live instance's command locator.
    pub(super) fn instance(&self) -> PublishedWorkflowInstanceRef {
        match &self.standing {
            Standing::Live(instance) => instance.clone(),
            Standing::Historical(posture) => panic!("the instance is live, not {posture:?}"),
        }
    }

    /// What was observed, whatever the read's standing.
    pub(super) fn observed(
        &self,
    ) -> (
        &CompositeCommitIdentity,
        WorthQueryWorkflowObservedState,
        &[String],
        usize,
    ) {
        (&self.commit, self.state, &self.paths, self.history)
    }
}

/// Reads the original instance's frontier and history at `snapshot`'s basis.
pub(super) fn frontier_at(
    snapshot: &WorthQuerySessionSnapshot<'_, '_, DocumentRetentionSchema, WorkflowOwnerProgram>,
    vocabulary: WorthQueryWorkflowVocabulary<'_, DocumentRetentionSchema, GovernedDocumentWorkflow>,
    original: &Original,
) -> Frontier {
    snapshot.with_request(|at| {
        let read = at
            .read_original_workflow::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                vocabulary,
                original,
                read_input(),
                read_input(),
            )
            .unwrap();
        let (standing, observation) = match read {
            WorthQueryWorkflowInstanceRead::Live {
                instance,
                observation,
            } => (Standing::Live(instance), observation),
            WorthQueryWorkflowInstanceRead::Historical {
                posture,
                observation,
            } => (Standing::Historical(posture), observation),
            WorthQueryWorkflowInstanceRead::Missing { .. } => {
                panic!("the original instance is disclosed to the reviewer")
            }
        };
        let history = at
            .read_workflow_history::<GovernedDocumentWorkflow, SubjectRead, AuditRead>(
                vocabulary,
                observation.target(),
                read_input(),
                read_input(),
                16,
                WorthQueryWorkflowIndexRead::PublishedOnly,
                None,
            )
            .unwrap();
        Frontier {
            commit: observation.selected_commit().clone(),
            standing,
            state: observation.state(),
            paths: observation.recorded_paths().to_vec(),
            history: history.entries().len(),
        }
    })
}

/// A fresh snapshot's frontier, read on the owned session's call worker.
/// Returns the snapshot's own commit beside what the workflow read saw.
pub(super) fn spawn_frontier(
    owned: &Owned,
    original: Original,
) -> WorthQueryPending<(CompositeCommitIdentity, Frontier), std::convert::Infallible> {
    owned.spawn_with_session(move |session| {
        let vocabulary = session
            .workflow_vocabulary::<GovernedDocumentWorkflow>()
            .expect("workflow registered");
        let snapshot = session.snapshot().unwrap();
        let pinned = snapshot.selected_commit().clone();
        (pinned, frontier_at(&snapshot, vocabulary, &original))
    })
}

/// Takes a call's value once its waker fired; the caller never blocks on
/// the call itself.
pub(super) fn woken<T: std::fmt::Debug, D: std::fmt::Debug>(pending: WorthQueryPending<T, D>) -> T {
    let (wake, woke) = channel();
    let mut pending = pending.with_waker(move || {
        let _ = wake.send(());
    });
    woke.recv_timeout(PATIENCE)
        .expect("the waker fires once the call completes");
    match pending.try_take() {
        Some(Ok(value)) => value,
        other => panic!("the woken call completed with its value: {other:?}"),
    }
}

/// Every `on_commit` call of one Home, tagged with the committed-basis version
/// the callback saw.
pub(super) struct CommitReports {
    reports: Receiver<WorthQueryCommittedBasisVersion>,
    _watch: WorthQueryCommitWatchGuard,
}

impl CommitReports {
    pub(super) fn watch(home: &Home, owned: &Owned) -> Self {
        let (report, reports) = channel();
        let observer = owned.clone();
        let watch = home
            .on_commit(move || {
                let _ = report.send(observer.committed_basis_version());
            })
            .unwrap();
        Self {
            reports,
            _watch: watch,
        }
    }

    /// Waits until the notifier reports the Home's current version: the
    /// commit that `step` landed (and any before it) was observed.
    pub(super) fn expect_reported(&self, home: &Home, step: &str) {
        let committed = home.committed_basis_version();
        loop {
            let seen = self
                .reports
                .recv_timeout(PATIENCE)
                .unwrap_or_else(|_| panic!("on_commit reports {step}"));
            if seen == committed {
                return;
            }
        }
    }
}
