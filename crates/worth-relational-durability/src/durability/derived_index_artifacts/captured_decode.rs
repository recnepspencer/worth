//! Read the supported cache tuple into request-owned original generation DATA.
use worth_relational_contracts::internal_port as contracts;
use super::funding::IndexCheckpointWork;
use super::{decode_budget::*, generation_decode_quote as quote, DerivedIndexCheckpointArtifacts};
use contracts::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use worth_relational_branch::internal_port::indexes::data::{DerivedIndexArtifacts, DerivedIndexArtifactsOwnerPort as _};
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_image::CapturedCheckpointGenerations;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) struct InspectedCarriedIndexSelections {
    pub(crate) format_version: u16,
    pub(crate) legacy_artifact_count: usize,
    pub(crate) bases: Vec<contracts::indexes::data::basis_checkpoint::IndexBasisCheckpoint>,
}

pub(crate) fn inspect_carried_bases(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<InspectedCarriedIndexSelections>, Stop> {
    IndexCheckpointWork::with_request(request, |work| {
        let mut fields = [None; 3];
        let mut index = 0;
        BorrowedPayload(payload)
            .elements(work, |raw| {
                let field = fields.get_mut(index).ok_or_else(
                    contracts::durability::canonical_commit_artifact::borrowed_payload::malformed,
                )?;
                *field = Some(raw);
                index += 1;
                Ok(())
            })
            .map_err(wire_stop)?;
        let [Some(legacy), Some(checkpoint), Some(format)] = fields else {
            return Err(super::corrupt("index tuple is incomplete").into());
        };
        let format_version: u16 = rmp_serde::from_slice(format.0)
            .map_err(|_| super::corrupt("invalid index tuple format"))?;
        if !DerivedIndexCheckpointArtifacts::supports_outer_format(
            format_version,
            !checkpoint.is_none(),
        ) {
            return Err(super::corrupt("unsupported index tuple").into());
        }
        if checkpoint.is_none() {
            return Ok(None);
        }
        let legacy_artifact_count = legacy
            .field("generations", work)
            .map_err(wire_stop)?
            .elements(work, |_| Ok(()))
            .map_err(wire_stop)?;
        if legacy_artifact_count != 0 {
            return Err(super::corrupt("mixed full and delta index tuple").into());
        }
        let count = checkpoint
            .field("generations", work)
            .map_err(wire_stop)?
            .elements(work, |_| Ok(()))
            .map_err(wire_stop)?;
        let mut budget = IndexDecodeBudget::new(work, count)?;
        budget
            .reserve(super::checkpoint_delta::quote_decode(checkpoint, work).map_err(wire_stop)?)?;
        let decoded: DerivedIndexCheckpointArtifacts = rmp_serde::from_slice(checkpoint.0)
            .map_err(|_| super::corrupt("invalid delta index generations"))?;
        Ok(Some(InspectedCarriedIndexSelections {
            format_version,
            legacy_artifact_count,
            bases: decoded.into_bases(),
        }))
    })
}

pub(crate) fn decode(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedCheckpointGenerations, Stop> {
    IndexCheckpointWork::with_request(request, |work| decode_funded(lane, payload, work))
}
fn decode_funded(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    payload: &[u8],
    work: &IndexCheckpointWork<'_>,
) -> Result<CapturedCheckpointGenerations, Stop> {
    let mut fields = [None; 3];
    let mut index = 0;
    BorrowedPayload(payload)
        .elements(work, |raw| {
            let field = fields.get_mut(index).ok_or_else(
                contracts::durability::canonical_commit_artifact::borrowed_payload::malformed,
            )?;
            *field = Some(raw);
            index += 1;
            Ok(())
        })
        .map_err(wire_stop)?;
    let [Some(legacy), Some(checkpoint), Some(format)] = fields else {
        return Err(super::corrupt("index tuple is incomplete").into());
    };
    let format: u16 = rmp_serde::from_slice(format.0)
        .map_err(|_| super::corrupt("invalid index tuple format"))?;
    if !DerivedIndexCheckpointArtifacts::supports_outer_format(format, !checkpoint.is_none()) {
        return Err(super::corrupt("unsupported index tuple").into());
    }
    let legacy_generations = legacy.field("generations", work).map_err(wire_stop)?;
    let count = if checkpoint.is_none() {
        legacy_generations
            .elements(work, |_| Ok(()))
            .map_err(wire_stop)?
    } else {
        if legacy_generations
            .elements(work, |_| Ok(()))
            .map_err(wire_stop)?
            != 0
        {
            return Err(super::corrupt("mixed full and delta index tuple").into());
        }
        checkpoint
            .field("generations", work)
            .map_err(wire_stop)?
            .elements(work, |_| Ok(()))
            .map_err(wire_stop)?
    };
    let mut budget = IndexDecodeBudget::new(work, count)?;
    let (generations, bases) = if checkpoint.is_none() {
        budget.reserve(quote::full(legacy_generations, work).map_err(wire_stop)?)?;
        let decoded: DerivedIndexArtifacts = rmp_serde::from_slice(legacy.0)
            .map_err(|_| super::corrupt("invalid full index generations"))?;
        (decoded.into_generations(), Vec::new())
    } else {
        budget
            .reserve(super::checkpoint_delta::quote_decode(checkpoint, work).map_err(wire_stop)?)?;
        let decoded: DerivedIndexCheckpointArtifacts = rmp_serde::from_slice(checkpoint.0)
            .map_err(|_| super::corrupt("invalid delta index generations"))?;
        let generations = decoded.readmit_controlled(lane, &mut budget)?;
        (generations, decoded.into_bases())
    };
    if generations.len() != count {
        return Err(super::corrupt("index generation count mismatch").into());
    }
    let mut output = work.array(count)?;
    for generation in generations {
        work.consult()?;
        output.push(Arc::new(generation))?;
    }
    Ok(CapturedCheckpointGenerations {
        generations: output.seal()?,
        bases,
        _funding: budget.finish()?,
    })
}

#[cfg(test)]
pub mod tests;
