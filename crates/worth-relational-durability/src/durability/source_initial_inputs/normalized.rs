//! The Complete image records original roles, independently of its schema union.
use contracts::branch::transition::sequence::RelationalBranchTransitionSequence;
use contracts::config::data::policies::MutationConfig;
use contracts::history::data::coordinates::CommitId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use serde::{Deserialize, Serialize};
use worth_relational_branch::internal_port::branch::transition::record::RelationalBranchTransition;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalOriginalRecord {
    pub(crate) position: PatchStreamPosition,
    pub(crate) commit: CommitId,
    pub(crate) payload_digest: [u8; 32],
    pub(crate) envelope_leaf: [u8; 32],
    pub(crate) prestate_schema: [u8; 32],
    pub(crate) result_schema: [u8; 32],
    pub(crate) mutation: MutationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransitionOriginalRecord {
    pub(crate) sequence: RelationalBranchTransitionSequence,
    pub(crate) follows: PatchStreamPosition,
    pub(crate) schema: [u8; 32],
    pub(crate) transition: RelationalBranchTransition,
    #[serde(deserialize_with = "required_association")]
    pub(crate) caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
    #[serde(deserialize_with = "required_digest")]
    pub(crate) artifact_digest: Option<[u8; 32]>,
}

fn required_association<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>, D::Error>{
    Option::deserialize(deserializer)
}
fn required_digest<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<[u8; 32]>, D::Error> {
    Option::deserialize(deserializer)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NormalizedOriginalInputs {
    pub(crate) canonical: Vec<CanonicalOriginalRecord>,
    pub(crate) transitions: Vec<TransitionOriginalRecord>,
    #[serde(skip)]
    pub(crate) memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Serialize)]
pub(crate) struct NormalizedOriginalInputsRef<'a, C: Serialize, T: Serialize> {
    pub(crate) canonical: &'a C,
    pub(crate) transitions: &'a T,
}
