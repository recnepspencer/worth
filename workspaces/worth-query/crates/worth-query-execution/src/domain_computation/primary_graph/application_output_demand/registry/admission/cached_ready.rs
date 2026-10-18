//! Select a retained native Ready for certification before successor admission retires it.
use super::super::{
    DemandRegistryState, DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint,
    WorthQueryOutputDemandKey,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::{
    InvalidationEditAdmission, WorthQueryOutputDemandDenial,
};

/// This returns scheduling custody only. The ordinary Ready path authenticates
/// its native witness and reads before it can settle or admit an actual refresh.
pub(super) fn predecessor(
    state: &DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,
    scope: WorthQueryOperationScopeEntityBinding,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
    // Select the newest row BEFORE testing Ready. A newer in-flight or stopped
    // row must never resurrect an older cached answer.
    let Some(key) = super::super::refreshed_rejoin::newest_matching_row(
        &state.records,
        requested,
        admission,
        |_, record| record.product_occurrence == occurrence && record.source_scope == Some(scope),
    )?
    else {
        return Ok(None);
    };
    state.charge_record_lookup(&key, admission)?;
    let work = key
        .comparison_work()
        .zip(requested.comparison_work())
        .and_then(|(old, new)| old.checked_add(new))
        .and_then(|keys| keys.checked_mul(2))
        .and_then(|keys| keys.checked_add(8))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(super::super::refreshed_rejoin::work_denial)?;
    admission
        .charge_external_work(work)
        .map_err(|_| super::super::refreshed_rejoin::work_denial())?;
    if key.producer != requested.producer
        || key.replacement_order(requested) != Some(std::cmp::Ordering::Less)
    {
        return Ok(None);
    }
    let record = &state.records[&key];
    if record.settlements.is_empty()
        || (record.prerequisites.is_empty() && record.checkpoint_prerequisites.is_none())
    {
        return Ok(None);
    }
    let ready = matches!(&record.state, DemandState::Output(output)
        if matches!(output.advancement(), WorthQueryOutputAdvancement::Idle)
            && matches!(&output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(completion))
                if completion.producer_mode().is_some()));
    // Restored rows without original producer mode retain their existing full
    // readmission fence. No equality or source-address surrogate can create it.
    Ok((ready && record.readmission_source.is_some()).then_some(key))
}
