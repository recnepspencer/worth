use super::super::retention::{
    arc_charge, arc_slice_charge, array_charge, btree_charge, sum, BridgeRetentionDenial as D,
    BridgeRetentionLedger, BridgeRetentionReservation,
};
use super::{
    clock_lane::BridgeManagedTemporalIntentRecord, BridgeManagedClockLane, BridgeManagedDueWake,
    BridgeManagedTemporalDenial, BridgeManagedTemporalDenialKind,
    BridgeManagedTemporalIntentIdentity,
};
use std::sync::{Arc, Mutex};

pub(super) fn reserve_clock(
    ledger: &Arc<BridgeRetentionLedger>,
    capacity: usize,
) -> Result<BridgeRetentionReservation, BridgeManagedTemporalDenial> {
    let charge = (|| {
        sum(&[
            arc_charge::<Mutex<BridgeManagedClockLane>>()?,
            arc_charge::<()>()?,
            // Reconstructive revocation snapshots retain one lane handle per clock.
            array_charge::<Arc<Mutex<BridgeManagedClockLane>>>(1)?,
            btree_charge::<Arc<str>, Arc<Mutex<BridgeManagedClockLane>>>(1)?,
            btree_charge::<BridgeManagedTemporalIntentIdentity, BridgeManagedTemporalIntentRecord>(
                capacity,
            )?,
            btree_charge::<
                worth_signal::facade::TemporalWakeId,
                BridgeManagedTemporalIntentIdentity,
            >(capacity)?,
            // Each admitted identity is at most 512 UTF-8 bytes. Include Arc
            // headers/alignment for intent and idempotency strings.
            identity_bytes(2)?
                .checked_mul(capacity as u64)
                .ok_or(D::BytesExhausted)?,
        ])
    })()
    .map_err(denial)?;
    ledger.reserve(1, capacity, charge).map_err(denial)
}

pub(super) fn reserve_binding(
    ledger: &Arc<BridgeRetentionLedger>,
) -> Result<BridgeRetentionReservation, BridgeManagedTemporalDenial> {
    let charge = sum(&[
        arc_charge::<super::contract::BridgeManagedClockLease>().map_err(denial)?,
        identity_bytes(3).map_err(denial)?,
    ])
    .map_err(denial)?;
    ledger.reserve(0, 0, charge).map_err(denial)
}

pub(super) fn reserve_due_output(
    ledger: &Arc<BridgeRetentionLedger>,
    capacity: usize,
) -> Result<Arc<BridgeRetentionReservation>, BridgeManagedTemporalDenial> {
    let charge = (|| {
        sum(&[
            arc_charge::<BridgeRetentionReservation>()?,
            array_charge::<BridgeManagedDueWake>(capacity)?,
            // Output can outlive lane membership and its identity allocations.
            identity_bytes(3)?
                .checked_mul(capacity as u64)
                .ok_or(D::BytesExhausted)?,
        ])
    })()
    .map_err(denial)?;
    let reservation = ledger.reserve(0, 0, charge).map_err(denial)?;
    Ok(Arc::new(reservation))
}

pub(super) fn denial(denial: D) -> BridgeManagedTemporalDenial {
    let kind = match denial {
        D::Closed => BridgeManagedTemporalDenialKind::ClosedClockBinding,
        D::Quarantined => BridgeManagedTemporalDenialKind::RetentionQuarantined,
        _ => BridgeManagedTemporalDenialKind::RetentionCapacityExhausted,
    };
    BridgeManagedTemporalDenial::new(
        kind,
        format!("Bridge managed-clock retention denied admission: {denial:?}"),
    )
}

fn identity_bytes(count: usize) -> Result<u64, D> {
    arc_slice_charge::<u8>(super::contract::MAXIMUM_IDENTITY_BYTES)?
        .checked_mul(count as u64)
        .ok_or(D::BytesExhausted)
}
