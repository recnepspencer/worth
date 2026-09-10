//! Allocation custody for selected payload clones and staged node roots.
use std::sync::Arc;

use super::{map_mutation_denial, NodeArena, RetainedNodeEditDenial, RetainedNodePayload};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Work,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};

pub(super) fn reserve_payload_draft(
    arena: &NodeArena,
    count: usize,
    ledger: &Arc<SignalConditionalRetentionLedger>,
    work: &mut Work,
) -> Result<SignalConditionalRetentionReservation, RetainedNodeEditDenial> {
    work.reserve_visits(4)
        .map_err(RetainedNodeEditDenial::Accounting)?;
    let payloads = Charge::capacity::<RetainedNodePayload>(count)
        .map_err(RetainedNodeEditDenial::Accounting)?;
    // Cloning selected payloads cannot exceed all reachable lane payloads.
    // Carried charges avoid a warm scan; the vector capacity is separate.
    let charge = [
        arena
            .hot
            .prepared_retained_charge()
            .map_err(map_mutation_denial)?,
        arena
            .warm
            .prepared_retained_charge()
            .map_err(map_mutation_denial)?,
        arena
            .cold
            .prepared_retained_charge()
            .map_err(map_mutation_denial)?,
    ]
    .into_iter()
    .try_fold(payloads, Charge::checked_add)
    .map_err(RetainedNodeEditDenial::Accounting)?;
    ledger
        .reserve(0, charge)
        .map_err(RetainedNodeEditDenial::Retention)
}

pub(super) fn reserve_staged_roots(
    ledger: &Arc<SignalConditionalRetentionLedger>,
    charge: Charge,
) -> Result<Arc<SignalConditionalRetentionReservation>, RetainedNodeEditDenial> {
    // reserve charges the reservation value itself. Its shared allocation also
    // owns the two Arc counters; admit those before constructing the Arc.
    let charge = charge
        .checked_add(Charge::capacity::<usize>(2).map_err(RetainedNodeEditDenial::Accounting)?)
        .map_err(RetainedNodeEditDenial::Accounting)?;
    let reservation = ledger
        .reserve(0, charge)
        .map_err(RetainedNodeEditDenial::Retention)?;
    Ok(Arc::new(reservation))
}
