mod header;
use super::HistoryAuthority;
use crate::history::{data::CommitId, RelationalCommitCatalog, RelationalCommitEnvelopeResidency};
use crate::runtime::RelationalRuntime;
pub use header::OutputOriginHeaderCorruptionForTest;
use std::sync::Arc;

impl HistoryAuthority<'_> {
    /// Corrupt an existing recorded envelope only for the enclosed court action.
    /// Restore the exact residency and catalog on return, panic, or early exit
    /// when history has not advanced.
    /// This supplies no validation proof; the corruption changes no current graph truth.
    /// Publishing inside the action invalidates the scope: it fails loudly and
    /// leaves the advanced history intact instead of restoring over new commits.
    pub fn with_corrupt_commit_envelope_for_test<R>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: CommitId,
        corruption: OutputOriginHeaderCorruptionForTest,
        action: impl FnOnce() -> R,
    ) -> R {
        let _restore = Restore::install(self.runtime, execution, commit, corruption);
        action()
    }
}

struct Restore<'a> {
    runtime: &'a RelationalRuntime,
    commit: CommitId,
    original: Arc<RelationalCommitEnvelopeResidency>,
    catalog: Option<RelationalCommitCatalog>,
    next_commit_id: u64,
    latest_commit: Option<CommitId>,
}
impl<'a> Restore<'a> {
    fn install(
        runtime: &'a RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commit: CommitId,
        corruption: OutputOriginHeaderCorruptionForTest,
    ) -> Self {
        let original = runtime.history.with_ledger_mut(|ledger| {
            Arc::clone(
                ledger
                    .commit_envelopes
                    .get(&commit)
                    .expect("court commit exists"),
            )
        });
        let frame = original
            .export_artifact(execution)
            .expect("court copies its actual recorded original frame");
        let admitted = frame
            .readmit_framed(execution)
            .expect("court original frame readmits under its genuine request")
            .into_positioned();
        let copied = Arc::clone(admitted.canonical_arc());
        drop(admitted);
        let mut envelope = Arc::try_unwrap(copied)
            .expect("the newly readmitted court original has unique body custody");
        envelope.corrupt_output_origin_header_for_test(corruption);
        assert_eq!(
            envelope.commit.commit_id, commit,
            "corruption preserves commit identity"
        );
        let original_payload = crate::branch::RelationalOriginalRootCommitEvidence::prepare(
            execution,
            Arc::new(envelope),
        )
        .expect("court's original request prepares the altered envelope before arming");
        let previous = runtime
            .history
            .commit_artifact(commit)
            .expect("court catalog entry exists");
        let replacement =
            crate::history::RelationalCommitArtifact::from_preencoded_with_descriptor(
                original_payload,
                previous.roots().clone(),
            )
            .expect("corruption preserves catalog identity")
            .prepare_catalog_owner(execution)
            .expect("court's original request funds catalog ownership");
        let replacement_cell = replacement.envelope_residency();
        let mut replacement_catalog = RelationalCommitCatalog::default();
        for artifact in runtime.history.commit_artifacts() {
            if artifact.commit_id() != commit {
                replacement_catalog.install_prepared_recovery(artifact);
            }
        }
        replacement_catalog
            .append_preencoded_recovery(replacement)
            .expect("corruption preserves catalog identity");
        let (original_catalog, next_commit_id, latest_commit) =
            runtime.history.with_ledger_mut(|ledger| {
                let next_commit_id = ledger.next_commit_id;
                let latest_commit = ledger
                    .commit_catalog
                    .latest_identity()
                    .map(|id| id.commit_id());
                (
                    std::mem::replace(&mut ledger.commit_catalog, replacement_catalog),
                    next_commit_id,
                    latest_commit,
                )
            });
        let restore = Restore {
            runtime,
            commit,
            original,
            catalog: Some(original_catalog),
            next_commit_id,
            latest_commit,
        };
        runtime.history.with_ledger_mut(|ledger| {
            ledger.commit_envelopes.insert(commit, replacement_cell);
        });
        restore
    }
}
impl Drop for Restore<'_> {
    fn drop(&mut self) {
        let movement = self.runtime.history.with_ledger_mut(|ledger| {
            let latest_commit = ledger.commit_catalog.latest_identity().map(|id| id.commit_id());
            if ledger.next_commit_id != self.next_commit_id || latest_commit != self.latest_commit {
                return Some(format!(
                    "recorded header corruption scope moved history: next_commit_id {} -> {}; latest_commit {:?} -> {:?}",
                    self.next_commit_id, ledger.next_commit_id, self.latest_commit, latest_commit,
                ));
            }
            ledger.commit_envelopes.insert(self.commit, Arc::clone(&self.original));
            ledger.commit_catalog = self.catalog.take().expect("restore once");
            None
        });
        if let Some(message) = movement {
            if std::thread::panicking() {
                eprintln!("{message}");
                std::process::abort();
            }
            panic!("{message}");
        }
    }
}
