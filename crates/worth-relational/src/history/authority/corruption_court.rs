mod header;
use super::HistoryAuthority;
use crate::history::{data::CommitId, RelationalCommitCatalog, RelationalCommitEnvelopeResidency};
use crate::runtime::RelationalRuntime;
pub use header::OutputOriginHeaderCorruptionForTest;
use std::sync::Arc;

impl HistoryAuthority<'_> {
    /// Corrupt an existing recorded envelope only for the enclosed court action.
    /// Restore the exact residency and catalog on return, panic, or early exit.
    /// This supplies no validation proof and never changes current graph truth.
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
        let mut catalog = runtime.history.with_ledger_mut(|ledger| {
            ledger
                .commit_catalog
                .retained_entries_except(&std::collections::BTreeSet::from([commit]))
        });
        catalog
            .append_checkpoint_recovery(Arc::new(envelope.clone()), None, Some(artifact.roots()))
            .expect("corruption preserves catalog identity");
        let original_catalog = runtime
            .history
            .with_ledger_mut(|ledger| std::mem::replace(&mut ledger.commit_catalog, catalog));
        let _restore = Restore {
            runtime: runtime,
            commit,
            original,
            catalog: Some(original_catalog),
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
        _restore
    }
}
impl Drop for Restore<'_> {
    fn drop(&mut self) {
        self.runtime.history.with_ledger_mut(|ledger| {
            ledger
                .commit_envelopes
                .insert(self.commit, Arc::clone(&self.original));
            ledger.commit_catalog = self.catalog.take().expect("restore once");
        });
    }
}
