//! Read the selected artifact through both original immutable Native roots.
use super::{
    RelationalRetainedEntitySelection, RelationalRetainedEntitySelectionDenial as Denial,
    RelationalRetainedEntitySelectionPreparationError as Error,
};
use crate::authority::commit::semantic_plan::PreparedRetainedEntityPayload;
use crate::branch::SelectedRelationalBranchState;
use crate::history::data::CanonicalRetainedEntitySelection;
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::CurrentPartitionAccess;
use std::sync::Arc;

pub(in crate::mvcc) fn prepare(
    runtime: &RelationalPreparationRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: &RelationalRetainedEntitySelection,
) -> Result<PreparedRetainedEntityPayload, Error> {
    let current = SelectedRelationalBranchState::from_admitted_basis(&input.transaction.basis);
    let historical = SelectedRelationalBranchState::from_admitted_basis(&input.historical);
    let source = crate::storage::residency::probe_head(
        execution,
        &historical.with_symbol_admission(&runtime.services.symbols),
        |source| {
            crate::storage::residency::probe_head(
                execution,
                &current.with_symbol_admission(&runtime.services.symbols),
                |target| read(input, source, target),
            )
            .map_err(|failure| storage(&input.transaction.basis, failure))?
            .map_err(Error::Admission)
        },
    )
    .map_err(|failure| storage(&input.historical, failure))?;
    source
}

fn read(
    input: &RelationalRetainedEntitySelection,
    historical: &dyn CurrentPartitionAccess,
    current: &dyn CurrentPartitionAccess,
) -> Result<PreparedRetainedEntityPayload, Denial> {
    let entity = input.entity;
    let old_region = historical
        .get_current_partition(entity.partition_id)
        .ok_or(Denial::HistoricalSourceUnavailable)?;
    let current_region = current
        .get_current_partition(entity.partition_id)
        .ok_or(Denial::IdentityChanged)?;
    let old = old_region
        .entity(entity)
        .filter(|slot| slot.is_live())
        .ok_or(Denial::HistoricalSourceUnavailable)?;
    let target = current_region
        .entity(entity)
        .filter(|slot| slot.is_live())
        .ok_or(Denial::IdentityChanged)?;
    let kind = old.kind_id().ok_or(Denial::HistoricalSourceUnavailable)?;
    if target.kind_id() != Some(kind) {
        return Err(Denial::IdentityChanged);
    }
    // Identity/derived headers remain current. This profile cannot transport
    // another lineage or structural meaning under an equal-looking entity ID.
    if old.extra().lineage_id != target.extra().lineage_id
        || old.extra().structural_fingerprint != target.extra().structural_fingerprint
    {
        return Err(Denial::UnsupportedIdentityMeaning);
    }
    let meaning = CanonicalRetainedEntitySelection::from_admitted(&input.historical, entity, kind)
        .ok_or(Denial::HistoricalSourceUnavailable)?;
    Ok(PreparedRetainedEntityPayload {
        _source: input.historical.clone(),
        meaning: Arc::new(meaning),
        payload: old.extra().authoritative_aspect_state.clone(),
    })
}

fn storage(
    basis: &crate::branch::AdmittedRelationalBranchBasis,
    failure: crate::storage::residency::RelationalResidencyDemandFailure,
) -> Error {
    Error::Preparation(crate::authority::commit::preparation::storage_read_denial(
        basis, failure,
    ))
}
