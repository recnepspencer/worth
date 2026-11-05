//! A compact subject retains the actual Native publication without dumping its graph.
use worth_relational::facade::mvcc::{CompanionPreflightStop, RelationalRequiredOutputPublication};
#[derive(Debug)]
pub enum HistoricalTransportDenial {
    Lookup(CompanionPreflightStop),
    PublicationCorrespondence(WorthQueryHistoryPublicationCorrespondenceMismatch),
}
/// Descriptive Query roles disagree with this selected Native publication.
#[derive(Clone)]
pub struct WorthQueryHistoryPublicationCorrespondenceMismatch {
    publication: RelationalRequiredOutputPublication,
}
impl WorthQueryHistoryPublicationCorrespondenceMismatch {
    pub(super) fn new(publication: RelationalRequiredOutputPublication) -> Self {
        Self { publication }
    }
    pub fn producer(&self) -> &str {
        self.publication.producer()
    }
    pub fn family(&self) -> &str {
        &self.publication.meaning().family
    }
    pub fn source(&self) -> worth_relational::facade::identity::EntityId {
        self.publication.meaning().source_scope
    }
}
impl std::fmt::Debug for WorthQueryHistoryPublicationCorrespondenceMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryPublicationCorrespondenceMismatch")
            .field("producer", &self.producer())
            .field("family", &self.family())
            .field("source", &self.source())
            .finish()
    }
}
impl From<CompanionPreflightStop> for HistoricalTransportDenial {
    fn from(cause: CompanionPreflightStop) -> Self {
        Self::Lookup(cause)
    }
}
