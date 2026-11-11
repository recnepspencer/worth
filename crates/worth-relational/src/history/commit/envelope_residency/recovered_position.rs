//! Admit an already-positioned source into its actual retained cell without Original byte allocation.
use super::*;
impl RelationalCommitEnvelopeResidency {
    pub(crate) fn bind_recovered_positioned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        positioned: &PositionedCanonicalCommit,
    ) -> Result<(), RelationalRecoveryStop> {
        let fail = || RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(positioned.position()),
            cause: Cause::IntegrityMismatch,
        };
        let (entry, source) = self.source_placement().ok_or_else(fail)?;
        if entry.position != positioned.position()
            || entry.commit != positioned.commit
            || entry.schema_version != positioned.schema_version
            || entry.association_discovery != positioned.envelope().association_discovery()
            || entry.root.schema_root()
                != &crate::schema::data::schema_authority_snapshot_digest_bytes(
                    &positioned.schema_authority,
                )
            || entry.payload_digest
                != RelationalCanonicalCommitArtifact::envelope_payload_digest_for_request(
                    positioned.position(),
                    positioned.envelope(),
                    request,
                )?
        {
            return Err(fail().into());
        }
        let (length, digest) = positioned
            .envelope()
            .authoritative_payload_observation_for_request(request)?;
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let same_source = match &state.backing {
            EnvelopeBacking::Journal {
                entry: actual,
                source: actual_source,
                ..
            } => actual.same_owner(&entry) && Arc::ptr_eq(actual_source, &source),
            EnvelopeBacking::SelectedCheckpoint {
                entry: actual,
                directory,
            } => actual.same_owner(&entry) && Arc::ptr_eq(&directory.source, &source),
            EnvelopeBacking::Memory { .. } => false,
        };
        if !same_source {
            return Err(fail().into());
        }
        state.position = Some(RetainedCanonicalPosition::from_positioned(positioned));
        state.payload_digest = Some(digest);
        state.canonical_payload_bytes = Some(length);
        state.lineage = Some(lineage_delta(positioned.envelope()));
        Ok(())
    }
}
