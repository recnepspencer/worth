//! Exact original bytes and original schema roles, without Home chronology.
use super::*;
use crate::durability::authority::checkpoint_complete::source::OriginalCanonicalCommit;
use branch::history::data::canonical_commit_envelope::original_inputs::CanonicalCommitEnvelopeOwnerPort as _;
use worth_execution::ExecutionRequestArrayBuilder;
use worth_relational_branch::internal_port as branch;
pub struct ReadmittedMemoryOriginals {
    pub(in crate::internal_port::durability::authority::checkpoint_complete::interim) frames:
        ExecutionArray<OriginalCanonicalCommit>,
    pub(in crate::internal_port::durability::authority::checkpoint_complete::interim) positioned:
        ExecutionArray<
            branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit,
        >,
}
impl FrozenMemoryOriginals {
    pub(in crate::internal_port::durability::authority::checkpoint_complete::interim) fn read_frames(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedMemoryOriginals, Denial> {
        let mut frames = ExecutionRequestArrayBuilder::allocate(*request, self.artifacts.len())
            .map_err(array_stop)?;
        let mut originals = ExecutionRequestArrayBuilder::allocate(*request, self.artifacts.len())
            .map_err(array_stop)?;
        for (offset, artifact) in self.artifacts.iter().enumerate() {
            let cell = artifact.envelope_residency();
            let original = crate::durability::canonical_commit_artifact::retained_frame_reads::require_original_artifact(&cell, request, None)
                .map_err(stop)?;
            let positioned = original.readmitted_body(request)
                .map_err(crate::durability::authority::checkpoint_complete::source::canonical_readmission_stop)?;
            let envelope = positioned.canonical_arc();
            if positioned.position().0 != offset as u64 + 1
                || envelope.commit.commit_id != artifact.commit_id()
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            match artifact.resident_envelope() {
                Some(actual) => {
                    if !crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::matches_positioned_payload_for_request(
                        &positioned, &actual, request).map_err(stop)? { return Err(Denial::OriginalAuthorityUnavailable); }
                }
                None => {
                    let (entry, _) = cell.source_placement().ok_or(Denial::OriginalAuthorityUnavailable)?;
                    if entry.commit != envelope.commit || entry.position != positioned.position()
                        || &entry.payload_digest != original.payload_digest().as_bytes() {
                        return Err(Denial::OriginalAuthorityUnavailable);
                    }
                }
            }
            let inputs = match artifact.original_inputs() {
                Some(inputs) => inputs.clone(),
                None => {
                    let recorded = envelope
                        .recorded_original_inputs()
                        .ok_or(Denial::OriginalAuthorityUnavailable)?;
                    let schemas = self
                        .schemas
                        .as_ref()
                        .ok_or(Denial::OriginalAuthorityUnavailable)?;
                    branch::history::commit::artifact::original_inputs::OriginalCanonicalInputs {
                        prestate_schema: schemas
                            .rows
                            .get(&recorded.prestate_schema)
                            .cloned()
                            .ok_or(Denial::OriginalAuthorityUnavailable)?,
                        result_schema: schemas
                            .rows
                            .get(&recorded.result_schema)
                            .cloned()
                            .ok_or(Denial::OriginalAuthorityUnavailable)?,
                        mutation: recorded.mutation.clone(),
                    }
                    .try_into()
                    .map_err(Denial::Request)?
                }
            };
            frames
                .push(OriginalCanonicalCommit {
                    position: positioned.position(),
                    artifact: artifact.clone(),
                    envelope: envelope.clone(),
                    inputs,
                    original_payload_digest: *original.payload_digest().as_bytes(),
                    original_frame: original.into_bytes(),
                })
                .map_err(array_stop)?;
            originals.push(positioned).map_err(array_stop)?;
        }
        Ok(ReadmittedMemoryOriginals {
            frames: frames.seal().map_err(array_stop)?,
            positioned: originals.seal().map_err(array_stop)?,
        })
    }
}
