//! Authenticated complete codec-disposition claims; cold readers re-admit them.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::schema_carrier::registrations::DurableEntitySchemaRegistration;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalCodecRetirementKind {
    pub source: DurableEntitySchemaRegistration,
    pub target: DurableEntitySchemaRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRequiredOutputCodecRetirement {
    pub publication: CommitId,
    pub meaning: crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    pub source: crate::branch::basis_descriptor::RelationalBranchBasisDescriptor,
    pub retiring: RelationalCommitReceipt,
    pub kinds: std::sync::Arc<Vec<CanonicalCodecRetirementKind>>,
    pub activation:
        std::sync::Arc<crate::schema::data::program_activation::RecoveredCheckpointProgramActivationUpdate>,
    #[serde(skip)]
    pub _funding: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
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

impl CanonicalCommitEnvelopeBody {
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
                        .any(|prior| prior.meaning.same_route(&event.meaning))
            })
    }
}

#[doc(hidden)]
pub trait CanonicalCommitEnvelopeBodyOwnerPort: Sized {
    fn codec_retirement_headers_valid(&self) -> bool;
}

impl CanonicalCommitEnvelopeBodyOwnerPort for CanonicalCommitEnvelopeBody {
    fn codec_retirement_headers_valid(&self) -> bool {
        self.codec_retirement_headers_valid()
    }
}
