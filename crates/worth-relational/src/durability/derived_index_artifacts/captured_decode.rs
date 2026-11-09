//! Read the supported cache tuple into request-owned original generation DATA.
use super::{decode_budget::*, generation_decode_quote as quote, DerivedIndexCheckpointArtifacts};
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use crate::durability::data::RelationalRecoveryStop as Stop;
use crate::indexes::data::{DerivedIndexArtifacts, DerivedIndexGeneration};
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};

pub(crate) struct CapturedCheckpointGenerations {
    pub(crate) generations: ExecutionArray<Arc<DerivedIndexGeneration>>,
    _funding: ExecutionArray<Option<ExecutionMemoryReservation>>,
}
pub(crate) fn decode(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<CapturedCheckpointGenerations, Stop> {
    let mut fields = [None; 3];
    let mut index = 0;
    BorrowedPayload(payload)
        .elements(request, |raw| {
            let field = fields.get_mut(index).ok_or_else(
                crate::durability::canonical_commit_artifact::borrowed_payload::malformed,
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
    let legacy_generations = legacy.field("generations", request).map_err(wire_stop)?;
    let count = if checkpoint.is_none() {
        legacy_generations
            .elements(request, |_| Ok(()))
            .map_err(wire_stop)?
    } else {
        if legacy_generations
            .elements(request, |_| Ok(()))
            .map_err(wire_stop)?
            != 0
        {
            return Err(super::corrupt("mixed full and delta index tuple").into());
        }
        checkpoint
            .field("generations", request)
            .map_err(wire_stop)?
            .elements(request, |_| Ok(()))
            .map_err(wire_stop)?
    };
    let mut budget = IndexDecodeBudget::new(*request, count)?;
    let generations = if checkpoint.is_none() {
        budget.reserve(quote::full(legacy_generations, request).map_err(wire_stop)?)?;
        let decoded: DerivedIndexArtifacts = rmp_serde::from_slice(legacy.0)
            .map_err(|_| super::corrupt("invalid full index generations"))?;
        decoded.into_generations()
    } else {
        budget.reserve(
            super::checkpoint_delta::quote_decode(checkpoint, request).map_err(wire_stop)?,
        )?;
        let decoded: DerivedIndexCheckpointArtifacts = rmp_serde::from_slice(checkpoint.0)
            .map_err(|_| super::corrupt("invalid delta index generations"))?;
        decoded.readmit_controlled(lane, &mut budget)?
    };
    if generations.len() != count {
        return Err(super::corrupt("index generation count mismatch").into());
    }
    let mut output = ExecutionRequestArrayBuilder::allocate(*request, count).map_err(array_stop)?;
    for generation in generations {
        request
            .consult()
            .map_err(|cause| Stop::Request(cause.into()))?;
        output.push(Arc::new(generation)).map_err(array_stop)?;
    }
    Ok(CapturedCheckpointGenerations {
        generations: output.seal().map_err(array_stop)?,
        _funding: budget.finish()?,
    })
}

#[cfg(test)]
mod tests;
