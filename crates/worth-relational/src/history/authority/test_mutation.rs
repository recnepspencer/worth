use super::HistoryAuthority;

impl<'runtime> HistoryAuthority<'runtime> {
    pub(crate) fn remove_commit_envelope_for_test(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> bool {
        let position = self.runtime.history.canonical_stream_position(commit_id);
        let removed = self.runtime.history.with_ledger_mut(|ledger| {
            let removed = ledger.commit_envelopes.remove(&commit_id).is_some();
            if removed && let Some(position) = position {
                ledger.patch_stream_index.remove(&position);
            }
            removed
        });
        if !removed {
            return false;
        }
        self.runtime.history.replace_catalog_from_legacy_for_test();
        true
    }

    pub(crate) fn remove_commit_envelope_preserving_patch_stream_position_for_test(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> bool {
        let removed = self
            .runtime
            .history
            .with_ledger_mut(|ledger| ledger.commit_envelopes.remove(&commit_id));
        self.runtime.history.replace_catalog_from_legacy_for_test();
        removed.is_some()
    }

    pub(crate) fn evict_commit_envelope_for_durable_recovery_test(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> bool {
        let removed = self.remove_commit_envelope_for_test(commit_id);
        self.runtime
            .history
            .remove_canonical_publication_route_for_test(commit_id);
        removed
    }

    pub(crate) fn evict_commit_envelope_preserving_patch_position_for_durable_recovery_test(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> bool {
        let removed =
            self.remove_commit_envelope_preserving_patch_stream_position_for_test(commit_id);
        self.runtime
            .history
            .remove_canonical_publication_route_for_test(commit_id);
        removed
    }

    pub(crate) fn tamper_commit_patch_for_test(
        &self,
        commit_id: crate::history::data::CommitId,
        mutate: impl FnOnce(&mut crate::publication::patch::data::CanonicalAuthoritativePatch),
    ) -> bool {
        let Some(envelope) = self.runtime.history.recorded_commit_envelope(commit_id) else {
            return false;
        };
        let mut replacement = envelope.as_ref().clone();
        mutate(&mut replacement.patch);
        let digest = replacement.authoritative_payload_digest().unwrap();
        self.runtime.history.with_ledger_mut(|ledger| {
            ledger.commit_envelopes.insert(
                commit_id,
                std::sync::Arc::new(crate::history::RelationalCommitEnvelopeResidency::resident(
                    std::sync::Arc::new(replacement),
                    digest,
                )),
            )
        });
        self.runtime.history.replace_catalog_from_legacy_for_test();
        true
    }

    pub(crate) fn tamper_commit_envelope_for_test(
        &self,
        commit_id: crate::history::data::CommitId,
        mutate: impl FnOnce(&mut crate::history::data::CanonicalCommitEnvelope),
    ) -> bool {
        let Some(envelope) = self.runtime.history.recorded_commit_envelope(commit_id) else {
            return false;
        };
        let mut replacement = envelope.as_ref().clone();
        if replacement.derived_index_artifacts().is_empty() {
            let generations = self
                .runtime
                .index_access()
                .generations_snapshot()
                .into_iter()
                .filter(|generation| generation.source_commit_id == commit_id)
                .collect::<Vec<_>>();
            replacement.derived_index_artifacts =
                crate::indexes::data::DerivedIndexArtifacts::new(generations);
        }
        mutate(&mut replacement);
        let digest = replacement.authoritative_payload_digest().unwrap();
        self.runtime.history.with_ledger_mut(|ledger| {
            ledger.commit_envelopes.insert(
                commit_id,
                std::sync::Arc::new(crate::history::RelationalCommitEnvelopeResidency::resident(
                    std::sync::Arc::new(replacement),
                    digest,
                )),
            )
        });
        self.runtime.history.replace_catalog_from_legacy_for_test();
        true
    }
}
