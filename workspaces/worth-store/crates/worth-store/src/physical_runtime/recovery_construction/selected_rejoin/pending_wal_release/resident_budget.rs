//! Co-live selected observations retained during the ordered history walk.

use worth_store_physical_format::PersistedPhysicalRecoveryProjection;
use worth_store_recovery_physics::VerifiedPendingWalReleaseCustody;

use crate::physical_runtime::{SharedRecoveryCheckpoint, StoreRecoveryBindingFreshnessSample};

use super::{
    controls::Controls, delta, head_v14::ObservedHeadV14, selection::Selection, Denial,
    SelectedControlMediaFingerprint,
};

pub(super) fn retained_with_heads(
    selected: &Selection,
    controls: &Controls,
    claim: &VerifiedPendingWalReleaseCustody,
    projection: &PersistedPhysicalRecoveryProjection,
    sample: &StoreRecoveryBindingFreshnessSample,
    checkpoint: &SharedRecoveryCheckpoint,
    heads: &ObservedHeadV14<'_>,
) -> Result<u64, Denial> {
    delta::retained_memory(selected, controls, claim, projection, sample, checkpoint)?
        .checked_add(heads.owned_heap_bytes().ok_or(Denial::BoundExceeded)?)
        .ok_or(Denial::BoundExceeded)
}

pub(super) fn extend_historical(
    controls: &mut SelectedControlMediaFingerprint,
    historical: SelectedControlMediaFingerprint,
    retained: u64,
) -> Result<(), Denial> {
    let available = delta::MAX_TRANSITION_MEMORY
        .checked_sub(retained)
        .ok_or(Denial::BoundExceeded)?;
    if !controls.try_extend_bounded(historical, available) {
        return Err(Denial::BoundExceeded);
    }
    Ok(())
}
