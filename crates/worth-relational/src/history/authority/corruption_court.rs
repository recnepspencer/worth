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
        commit: CommitId,
        corruption: OutputOriginHeaderCorruptionForTest,
        action: impl FnOnce() -> R,
    ) -> R {
        let _restore = Restore::install(self.runtime, commit, corruption);
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
        let mut envelope = runtime
            .history
            .recorded_commit_envelope(commit)
            .expect("court envelope is resident")
            .as_ref()
            .clone();
        envelope.corrupt_output_origin_header_for_test(corruption);
        assert_eq!(
            envelope.commit.commit_id, commit,
            "corruption preserves commit identity"
        );
        let payload = Arc::from(
            envelope
                .encode_authoritative_payload()
                .expect("court envelope encodes"),
        );
        let artifact = runtime
            .history
            .commit_artifact(commit)
            .expect("court catalog entry exists");
        let (original_catalog, next_commit_id, latest_commit) =
            runtime.history.with_ledger_mut(|ledger| {
                let mut catalog = ledger
                    .commit_catalog
                    .retained_entries_except(&std::collections::BTreeSet::from([commit]));
                catalog
                    .append_checkpoint_recovery(
                        Arc::new(envelope.clone()),
                        None,
                        Some(artifact.roots()),
                    )
                    .expect("corruption preserves catalog identity");
                let next_commit_id = ledger.next_commit_id;
                let latest_commit = ledger
                    .commit_catalog
                    .latest_identity()
                    .map(|id| id.commit_id());
                (
                    std::mem::replace(&mut ledger.commit_catalog, catalog),
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
            ledger.commit_envelopes.insert(
                commit,
                Arc::new(RelationalCommitEnvelopeResidency::resident(
                    Arc::new(envelope),
                    payload,
                )),
            );
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
