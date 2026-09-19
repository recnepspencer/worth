use std::sync::Arc;

use crate::data::retained_storage::{
    arc_allocation_charge, btree_structure_charge, ordered_index_charge,
    RetainedStorageCharge as Charge, RetainedStoragePreparationDenial,
    SignalConditionalRetentionDenial, SignalConditionalRetentionReservation,
};
use crate::data::temporal::{
    ClockTick, ReadyTemporalWake, RetiredTemporalWake, ScheduledTemporalWake, TemporalWakeId,
    TemporalWakeOwner, WakeOrdinal,
};

use super::partition::SignalConditionalTemporalCell;
use super::registry::SignalConditionalTemporalPartitionId;
use super::SignalConditionalTemporalPartitionDenial as Denial;

/// Concrete upper bound for the exclusively owned indexed scheduler. Ordinary
/// mutations never fork its PersistentOrdMaps: their storage remains std BTreeMap.
/// Both scheduled and ready representations receive the entire W bound, covering
/// their transition overlap. Every due bucket is charged as an im map with one
/// entry; W such bounds also dominate fewer buckets containing W total entries.
/// Manual ownership uses one im bucket. Supersession/retirement briefly retain
/// one retired receipt before the service forgets it. Wake records are inline.
pub(super) fn partition_charge(wakes: usize) -> Result<Charge, RetainedStoragePreparationDenial> {
    type OrdinalIndex = im::OrdMap<WakeOrdinal, TemporalWakeId>;
    arc_allocation_charge::<SignalConditionalTemporalCell>()?
        .checked_add(arc_allocation_charge::<
            SignalConditionalRetentionReservation,
        >()?)?
        // Each live row covers one registry node and an empty root. Resetting
        // the map after its last row disappears prevents an uncharged empty root.
        .checked_add(btree_structure_charge::<
            SignalConditionalTemporalPartitionId,
            Arc<SignalConditionalTemporalCell>,
        >(1)?)?
        .checked_add(btree_structure_charge::<
            TemporalWakeId,
            ScheduledTemporalWake,
        >(wakes)?)?
        .checked_add(btree_structure_charge::<ClockTick, OrdinalIndex>(wakes)?)?
        .checked_add(ordered_index_charge::<WakeOrdinal, TemporalWakeId>(1)?.checked_mul(wakes)?)?
        .checked_add(btree_structure_charge::<TemporalWakeId, ReadyTemporalWake>(
            wakes,
        )?)?
        .checked_add(btree_structure_charge::<WakeOrdinal, TemporalWakeId>(
            wakes,
        )?)?
        .checked_add(btree_structure_charge::<TemporalWakeOwner, OrdinalIndex>(
            1,
        )?)?
        .checked_add(ordered_index_charge::<WakeOrdinal, TemporalWakeId>(wakes)?)?
        .checked_add(btree_structure_charge::<TemporalWakeId, RetiredTemporalWake>(1)?)
}

pub(super) fn map_retention_denial(denial: SignalConditionalRetentionDenial) -> Denial {
    match denial {
        SignalConditionalRetentionDenial::Closed => Denial::RetentionClosed,
        SignalConditionalRetentionDenial::CapacityExhausted => Denial::RetentionCapacityExhausted,
        SignalConditionalRetentionDenial::Poisoned
        | SignalConditionalRetentionDenial::InvalidTransfer => Denial::RetentionQuarantined,
    }
}
