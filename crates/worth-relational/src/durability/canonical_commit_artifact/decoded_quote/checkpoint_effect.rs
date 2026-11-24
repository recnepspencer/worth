//! Image effect DATA uses the original semantic and creation allocation owners.
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResultFieldsPort as _;
use super::*;
use crate::durability::canonical_commit_artifact::decoded_quote::checkpoint_effect_data::CheckpointEffectData;
use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use crate::transactions::data::outcomes::commit_semantic_result::RelationalCommitSemanticResult;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

pub(crate) fn decode_checkpoint_effect(
    semantic: &[u8],
    creations: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<CheckpointEffectData, Refusal> {
    let semantic = BorrowedPayload(semantic);
    let creations = BorrowedPayload(creations);
    let quote = super::semantic::quote(semantic, request)?;
    let (mut validation, leaves) = validation::prepare_semantic_readmission(semantic, request)?;
    let creations_quote =
        vector::<PublishedAuthoritativeRecordPatch>(creations, request, |patch| {
            super::patch::record(patch, request)
        })?;
    let transient = quote
        .transient
        .checked_add(creations_quote.transient)
        .ok_or_else(malformed)?;
    let _scratch = reserve(transient, request)?;
    let semantic_memory = reserve(
        quote.retained.checked_sub(leaves).ok_or_else(malformed)?,
        request,
    )?;
    let memory = reserve(creations_quote.retained, request)?;
    crate::durability::data::recovery_stop::account_work(
        request,
        semantic
            .0
            .len()
            .checked_add(creations.0.len())
            .ok_or_else(malformed)? as u64,
    )
    .map_err(|cause| match cause {
        crate::durability::data::recovery_stop::RelationalRecoveryStop::Content(cause) => {
            Refusal::Content(cause)
        }
        crate::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause) => {
            Refusal::Request(cause)
        }
        crate::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(_)
        | crate::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(_)
        | crate::durability::data::recovery_stop::RelationalRecoveryStop::Durability(_) => {
            malformed()
        }
    })?;
    let mut semantic: RelationalCommitSemanticResult =
        rmp_serde::from_slice(semantic.0).map_err(|_| malformed())?;
    (*semantic.owner_decoded_memory_mut()) = semantic_memory;
    validation
        .attach(semantic.owner_validation())
        .map_err(|_| malformed())?;
    let creations = rmp_serde::from_slice(creations.0).map_err(|_| malformed())?;
    Ok(CheckpointEffectData {
        semantic: Arc::new(semantic),
        creations,
        _memory: memory,
    })
}

fn reserve(
    bytes: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionMemoryReservation>, Refusal> {
    request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))
}
