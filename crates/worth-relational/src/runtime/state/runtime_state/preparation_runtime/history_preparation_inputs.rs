//! Select the history and preparation owners for the selected commit.
use super::RelationalPreparationRuntime;
use crate::authority::commit::pipeline::candidate_preparation::CandidateHistoryInputs;
use crate::authority::commit::pipeline::history_preparation::HistoryPreparationInputs;

impl RelationalPreparationRuntime {
    pub(crate) fn candidate_history_inputs(&self) -> CandidateHistoryInputs<'_> {
        CandidateHistoryInputs {
            runtime_identity: self.runtime_instance_id(),
            publication: &self.publication_owner,
            history: &self.history,
            prepare: HistoryPreparationInputs {
                admission: self.into(),
                continuity: self.into(),
                descriptor_semantics: self
                    .config
                    .schema
                    .descriptor_semantics_policy
                    .current_write_version(),
            },
        }
    }
}
