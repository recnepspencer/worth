//! Canonical negative membership of a selected producer route.
use super::super::{CanonicalCommitEnvelopeBody, CommitId, Deserialize, Serialize};
use crate::identity::data::VersionId;
use worth_execution::ExecutionMemoryReservation;

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalHistoricalOutputAbsence {
    pub meaning: crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    pub publication: CommitId,
    pub generation: VersionId,
    #[serde(skip)]
    pub _funding: Option<ExecutionMemoryReservation>,
}
impl PartialEq for CanonicalHistoricalOutputAbsence {
    fn eq(&self, other: &Self) -> bool {
        (&self.meaning, self.publication, self.generation)
            == (&other.meaning, other.publication, other.generation)
    }
}
impl Eq for CanonicalHistoricalOutputAbsence {}
impl CanonicalCommitEnvelopeBody {
    pub(crate) fn historical_absent_routes(
        &self,
    ) -> impl Iterator<Item = &CanonicalHistoricalOutputAbsence> {
        self.historical_output_selection
            .iter()
            .flat_map(|event| event.absences.iter())
    }
}
