//! Admit the concrete delta headers before Serde, then quote each reconstructed entry.
use super::super::generation_decode_quote::{add, roster, tree, vector};
use super::*;
use crate::durability::canonical_commit_artifact::{
    borrowed_payload::BorrowedPayload, RelationalCanonicalCommitArtifactRefusal as Refusal,
};
use worth_execution::ExecutionRequest;
pub(in super::super) fn quote_decode(
    raw: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let mut bytes = 0;
    let count = raw.field("generations", request)?.elements(request, |entry| {
        entry.fields(request, |name, value| {
            match name {
                // Header strings remain alive while their output copies exist.
                "source_branch_id" | "applicability_branch_id" => add(&mut bytes, (value.text_length(request)? as u64).checked_mul(2).ok_or_else(crate::durability::canonical_commit_artifact::borrowed_payload::malformed)?)?,
                "changed_bytes" => {
                    let mut octets = value.octets(request)?;
                    let count = octets.len(); octets.skip(count)?; octets.finish()?;
                    add(&mut bytes, vector::<u8>(count)?)?;
                }
                _ => {}
            }
            Ok(())
        })
    })?;
    add(&mut bytes, vector::<CheckpointGeneration>(count)?)?;
    // Carried exact-root selections are decoded with the same payload.
    let bases = raw.field("bases", request)?.elements(request, |basis| {
        basis.fields(request, |name, value| {
            match name {
                "branch" => add(&mut bytes, value.text_length(request)? as u64)?,
                "empty" => {
                    let count = value.elements(request, |_| Ok(()))?;
                    add(&mut bytes, vector::<DerivedIndexId>(count)?)?;
                }
                "bindings" => {
                    let count = value.elements(request, |_| Ok(()))?;
                    add(
                        &mut bytes,
                        vector::<(DerivedIndexId, DerivedIndexGenerationId)>(count)?,
                    )?;
                }
                _ => {}
            }
            Ok(())
        })
    })?;
    add(
        &mut bytes,
        vector::<crate::indexes::data::IndexBasisCheckpoint>(bases)?,
    )?;
    let ids = raw
        .field("generation_ids", request)?
        .elements(request, |_| Ok(()))?;
    add(&mut bytes, vector::<DerivedIndexGenerationId>(ids)?)?;
    // The reader's manifest comparison makes a second temporary identity Vec.
    add(&mut bytes, vector::<DerivedIndexGenerationId>(count)?)?;
    add(&mut bytes, roster(count)?)?;
    add(&mut bytes, tree::<DerivedIndexGenerationId, ()>(count)?)?;
    add(
        &mut bytes,
        tree::<
            DerivedIndexId,
            (
                DerivedIndexGenerationId,
                (
                    Cow<'_, [u8]>,
                    Option<worth_execution::ExecutionMemoryReservation>,
                ),
            ),
        >(count)?,
    )?;
    Ok(bytes)
}
