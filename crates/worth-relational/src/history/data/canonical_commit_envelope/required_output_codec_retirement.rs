//! Authenticated complete codec-disposition claims; cold readers re-admit them.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalCodecRetirementKind {
    pub(crate) source: crate::durability::data::DurableEntitySchemaRegistration,
    pub(crate) target: crate::durability::data::DurableEntitySchemaRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CanonicalRequiredOutputCodecRetirement {
    pub(crate) publication: CommitId,
    pub(crate) meaning: crate::mvcc::RequiredOutputPublicationMeaning,
    pub(crate) source: crate::branch::RelationalBranchBasisDescriptor,
    pub(crate) retiring: RelationalCommitReceipt,
    pub(crate) kinds: std::sync::Arc<Vec<CanonicalCodecRetirementKind>>,
    pub(crate) activation:
        std::sync::Arc<crate::runtime::RecoveredCheckpointProgramActivationUpdate>,
    #[serde(skip)]
    pub(crate) _funding: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl PartialEq for CanonicalRequiredOutputCodecRetirement {
    fn eq(&self, other: &Self) -> bool {
        self.publication == other.publication
            && self.meaning == other.meaning
            && self.source == other.source
            && self.retiring == other.retiring
            && self.kinds == other.kinds
            && self.activation == other.activation
    }
}
impl Eq for CanonicalRequiredOutputCodecRetirement {}

impl CanonicalCommitEnvelope {
    pub(crate) fn codec_retirement_headers_valid(&self) -> bool {
        self.required_output_codec_retirements
            .iter()
            .enumerate()
            .all(|(index, event)| {
                self.required_output_origin.is_none()
                    && self.authority_kind == CanonicalCommitAuthorityKind::VersionedTransaction
                    && self.schema_transition.is_some()
                    && self.is_unmerged_output_publication_step()
                    && self.commit.parents.len() == 1
                    && event.retiring == self.commit
                    && event.publication != self.commit.commit_id
                    && !event.meaning.family.is_empty()
                    && !event.kinds.is_empty()
                    && event.source.branch_id() == &self.branch_context
                    && self
                        .branch_cell_checkpoint
                        .as_ref()
                        .map(|cell| &cell.observation)
                        == Some(event.source.reference())
                    && event
                        .source
                        .reference()
                        .target()
                        .as_basis()
                        .map(|basis| CommitId(basis.selected_commit_id()))
                        == self.commit.parents.first().copied()
                    && !self.required_output_codec_retirements[..index]
                        .iter()
                        .any(|prior| {
                            prior.publication == event.publication
                                || prior.meaning.same_route(&event.meaning)
                        })
            })
    }
}
