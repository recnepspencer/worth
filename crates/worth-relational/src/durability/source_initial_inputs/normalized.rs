//! The Complete image records original roles, independently of its schema union.
use crate::branch::{RelationalBranchTransition, RelationalBranchTransitionSequence};
use crate::config::data::MutationConfig;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use serde::{Deserialize, Serialize};

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
    pub(crate) caller_association: Option<crate::history::data::RelationalCallerAssociation>,
    #[serde(deserialize_with = "required_digest")]
    pub(crate) artifact_digest: Option<[u8; 32]>,
}

fn required_association<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<crate::history::data::RelationalCallerAssociation>, D::Error> {
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
