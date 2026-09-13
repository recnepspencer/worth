use super::{
    BlobPublicationEvidence, BlobPublicationObservationSet, BlobPublicationObservedSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationObservationAdmission {
    admitted_source: BlobPublicationObservedSource,
}

impl BlobPublicationObservationAdmission {
    pub fn admit_observations(observations: BlobPublicationObservationSet) -> Self {
        let sources = observations.into_sources();
        let admitted_source = sources
            .torn_publication
            .or(sources.persisted_crash_edge)
            .or(sources.backend_residue)
            .or(sources.live_ack_memory)
            .or(sources.log_only)
            .or(sources.insufficient_persisted_evidence)
            .unwrap_or_else(|| {
                BlobPublicationObservedSource::insufficient_persisted_evidence(
                    "blob-publication:no-observations",
                )
            });
        Self { admitted_source }
    }

    pub(crate) fn into_evidence(self) -> BlobPublicationEvidence {
        self.admitted_source.into_evidence()
    }
}
