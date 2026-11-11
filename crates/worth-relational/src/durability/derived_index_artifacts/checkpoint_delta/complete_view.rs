//! Request-funded format-3 emission over original generation owners, never a delta.
use super::*;
use crate::durability::artifact_payload_emission::request::encode_named;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::indexes::data::IndexBasisCheckpoint;
use serde::ser::{SerializeSeq, SerializeStruct, Serializer};
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder,
};

/// The Complete writer's borrowed wire twin of [`DerivedIndexCheckpointArtifacts`].
///
/// Every generation is its own full row, so the reader needs no base. A basis
/// row is emitted only when the writer retains the root image that proves it.
pub(crate) struct CompleteIndexCheckpointView<'a> {
    generations: &'a [Arc<DerivedIndexGeneration>],
    encoded: ExecutionArray<(ExecutionImmutableBytes, [u8; 32])>,
    bases: &'a [IndexBasisCheckpoint],
    retained: &'a dyn Fn(&IndexBasisCheckpoint) -> bool,
    retained_count: usize,
}

impl<'a> CompleteIndexCheckpointView<'a> {
    pub(crate) fn capture(
        generations: &'a [Arc<DerivedIndexGeneration>],
        bases: &'a [IndexBasisCheckpoint],
        retained: &'a dyn Fn(&IndexBasisCheckpoint) -> bool,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Stop> {
        let mut encoded = ExecutionRequestArrayBuilder::allocate(*request, generations.len())
            .map_err(array_stop)?;
        for generation in generations {
            let entries = encode_named(&[], &generation.entries, request)?;
            let header = encode_named(
                &[],
                &CheckpointDigestHeader::from_generation(generation),
                request,
            )?;
            for _ in entries.bytes().chunks(64 * 1024) {
                consult(request)?;
            }
            let digest = digest_bytes(header.bytes(), entries.bytes())?;
            encoded.push((entries, digest)).map_err(array_stop)?;
        }
        let mut retained_count = 0_usize;
        for basis in bases {
            consult(request)?;
            if retained(basis) {
                retained_count += 1;
            }
        }
        Ok(Self {
            generations,
            encoded: encoded.seal().map_err(array_stop)?,
            bases,
            retained,
            retained_count,
        })
    }
}

fn consult(request: &ExecutionRequest<'_, '_>) -> Result<(), Stop> {
    request
        .consult()
        .map_err(|cause| Stop::Request(cause.into()))
}
fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> Stop {
    super::super::decode_budget::array_stop(cause)
}

impl Serialize for CompleteIndexCheckpointView<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut row = serializer.serialize_struct("DerivedIndexCheckpointArtifacts", 4)?;
        row.serialize_field(
            "format_version",
            &DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
        )?;
        row.serialize_field("generation_ids", &Identities(self.generations))?;
        row.serialize_field("generations", &Rows(self))?;
        row.serialize_field("bases", &RetainedBases(self))?;
        row.end()
    }
}

struct Identities<'a>(&'a [Arc<DerivedIndexGeneration>]);
impl Serialize for Identities<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.iter().map(|generation| generation.generation_id))
    }
}

struct Rows<'a, 'b>(&'a CompleteIndexCheckpointView<'b>);
impl Serialize for Rows<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.generations.iter().zip(self.0.encoded.iter()).map(
            |(generation, (entries, digest))| Row {
                generation,
                entries: entries.bytes(),
                digest,
            },
        ))
    }
}

struct RetainedBases<'a, 'b>(&'a CompleteIndexCheckpointView<'b>);
impl Serialize for RetainedBases<'_, '_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut rows = serializer.serialize_seq(Some(self.0.retained_count))?;
        for basis in self.0.bases.iter().filter(|basis| (self.0.retained)(basis)) {
            rows.serialize_element(basis)?;
        }
        rows.end()
    }
}

/// Field names and order are those of the derived [`CheckpointGeneration`] row.
struct Row<'a> {
    generation: &'a DerivedIndexGeneration,
    entries: &'a [u8],
    digest: &'a [u8; 32],
}
impl Serialize for Row<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let generation = self.generation;
        let mut row = serializer.serialize_struct("CheckpointGeneration", 13)?;
        row.serialize_field("generation_id", &generation.generation_id)?;
        row.serialize_field("index_id", &generation.index_id)?;
        row.serialize_field("source_commit_id", &generation.source_commit_id)?;
        row.serialize_field("source_branch_id", &generation.source_branch_id)?;
        row.serialize_field(
            "applicability_branch_id",
            &generation.applicability.branch_id,
        )?;
        row.serialize_field("version_id", &generation.applicability.version_id)?;
        row.serialize_field("schema_version", &generation.applicability.schema_version)?;
        row.serialize_field("status", &generation.status)?;
        row.serialize_field("base_generation_id", &None::<DerivedIndexGenerationId>)?;
        row.serialize_field("shared_prefix_bytes", &0_u64)?;
        row.serialize_field("shared_suffix_bytes", &0_u64)?;
        row.serialize_field("changed_bytes", self.entries)?;
        row.serialize_field("full_digest", self.digest)?;
        row.end()
    }
}

#[cfg(test)]
mod tests;
