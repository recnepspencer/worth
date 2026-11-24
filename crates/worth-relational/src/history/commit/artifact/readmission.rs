//! Match retained canonical commitments independently of selected placement.
impl super::RelationalCommitArtifact {
    /// Compare a recovered body against the retained commitment without loading it.
    pub(crate) fn matches_envelope(
        &self,
        envelope: &crate::history::data::CanonicalCommitEnvelope,
    ) -> Result<bool, super::RelationalCommitArtifactDenial> {
        if let Some(resident) = self.resident_envelope() {
            return Ok(resident.as_ref() == envelope);
        }
        let entry = self
            .envelope_residency()
            .journal_index()
            .ok_or(super::RelationalCommitArtifactDenial::RootLinkage)?;
        let digest =
            crate::history::commit::payload_commitment::digest_envelope(entry.position, envelope)
                .map_err(|_| super::RelationalCommitArtifactDenial::CanonicalPayloadEncoding)?;
        Ok(entry.commit == envelope.commit
            && entry.schema_version == envelope.schema_version
            && entry.payload_digest == digest)
    }
}
