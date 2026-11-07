//! Exact original bytes and original schema roles, without Home chronology.
use super::*;
use crate::durability::authority::checkpoint_complete::source::OriginalCanonicalCommit;
use worth_execution::ExecutionRequestArrayBuilder;
pub(in crate::durability::authority::checkpoint_complete::interim) struct ReadmittedMemoryOriginals
{
    pub(in crate::durability::authority::checkpoint_complete::interim) frames:
        ExecutionArray<OriginalCanonicalCommit>,
    pub(in crate::durability::authority::checkpoint_complete::interim) positioned:
        ExecutionArray<crate::history::data::PositionedCanonicalCommit>,
}
impl FrozenMemoryOriginals {
    pub(in crate::durability::authority::checkpoint_complete::interim) fn read_frames(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedMemoryOriginals, Denial> {
        let mut frames = ExecutionRequestArrayBuilder::allocate(*request, self.artifacts.len())
            .map_err(array_stop)?;
        let mut originals = ExecutionRequestArrayBuilder::allocate(*request, self.artifacts.len())
            .map_err(array_stop)?;
        for (offset, artifact) in self.artifacts.iter().enumerate() {
            let cell = artifact.envelope_residency();
            let original = cell
                .require_original_artifact(request, None)
                .map_err(stop)?;
            let positioned = original.readmit_framed(request)
                .map_err(crate::durability::authority::checkpoint_complete::source::canonical_readmission_stop)?.into_positioned();
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
                    crate::history::OriginalCanonicalInputs {
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
