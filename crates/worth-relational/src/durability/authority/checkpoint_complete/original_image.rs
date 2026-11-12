//! Borrow the frozen original prefix; serialization never reads a live owner.
use super::source::OriginalCanonicalCommit;
use crate::durability::source_initial_inputs::normalized::{
    CanonicalOriginalRecord, NormalizedOriginalInputsRef,
};
use crate::runtime::OriginalBranchSchemaInputs;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

pub(super) fn emit(
    source: &super::source::FrozenOriginalInputs,
    commits: &[OriginalCanonicalCommit],
    request: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionImmutableBytes, super::Denial> {
    let original = &source.branches;
    let prior = source.base.as_ref().map(|base| base.state.normalized());
    let mut leaves =
        worth_execution::ExecutionRequestArrayBuilder::allocate(*request, commits.len())
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    for commit in commits {
        let leaf = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::canonical_leaf_from_verified_frame(
            commit.original_frame.bytes(), request,
        ).map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        leaves
            .push(leaf)
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    }
    let leaves = leaves
        .seal()
        .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
    let canonical = Canonical(commits, &leaves, prior);
    for entry in original.transitions() {
        let schema = entry
            .schema
            .recorded_image(request)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        if entry.positioned.schema_carrier() != Some(schema.carrier_digest()) {
            return Err(super::Denial::OriginalAuthorityUnavailable);
        }
    }
    let transitions = Transitions(original);
    crate::durability::artifact_payload_emission::request::encode_named(
        &[],
        &NormalizedOriginalInputsRef {
            canonical: &canonical,
            transitions: &transitions,
        },
        request,
    )
    .map_err(crate::durability::checkpoint_history::capture::capture_stop)
}

struct Canonical<'a>(
    &'a [OriginalCanonicalCommit],
    &'a [[u8; 32]],
    Option<&'a crate::durability::source_initial_inputs::normalized::NormalizedOriginalInputs>,
);
impl Serialize for Canonical<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(
            self.0.len() + self.2.map_or(0, |prior| prior.canonical.len()),
        ))?;
        if let Some(prior) = self.2 {
            for row in &prior.canonical {
                rows.serialize_element(row)?;
            }
        }
        for (original, leaf) in self.0.iter().zip(self.1) {
            let inputs = original
                .envelope
                .recorded_original_inputs()
                .ok_or_else(|| {
                    serde::ser::Error::custom("original canonical attribution missing")
                })?;
            rows.serialize_element(&CanonicalOriginalRecord {
                position: original.position,
                commit: original.envelope.commit.commit_id,
                payload_digest: original.original_payload_digest,
                envelope_leaf: *leaf,
                prestate_schema: inputs.prestate_schema,
                result_schema: inputs.result_schema,
                mutation: inputs.mutation.clone(),
            })?;
        }
        rows.end()
    }
}

struct Transitions<'a>(&'a OriginalBranchSchemaInputs);
impl Serialize for Transitions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.transitions().len()))?;
        for original in self.0.transitions() {
            let schema = original.positioned.schema_carrier().ok_or_else(|| {
                serde::ser::Error::custom("transition schema attribution missing")
            })?;
            rows.serialize_element(&TransitionOriginalRecordRef {
                sequence: original.positioned.sequence(),
                follows: original.positioned.follows(),
                schema,
                transition: original.positioned.transition(),
                caller_association: original.positioned.caller_association(),
                artifact_digest: original.positioned.recorded_artifact_digest(),
            })?;
        }
        rows.end()
    }
}

#[derive(Serialize)]
struct TransitionOriginalRecordRef<'a> {
    sequence: crate::branch::RelationalBranchTransitionSequence,
    follows: crate::publication::patch::data::PatchStreamPosition,
    schema: [u8; 32],
    transition: &'a crate::branch::RelationalBranchTransition,
    caller_association: Option<&'a crate::history::data::RelationalCallerAssociation>,
    artifact_digest: Option<[u8; 32]>,
}
