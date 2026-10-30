use std::sync::atomic::{AtomicU64, Ordering};

use super::{RelationalAuthorizationObservationIdentity, RelationalAuthorizationObservationPlan};

static NEXT_AUTHORIZATION_OBSERVATION_ORDINAL: AtomicU64 = AtomicU64::new(1);

pub(super) fn mint_observation_identity(
    plan: &RelationalAuthorizationObservationPlan,
) -> Option<RelationalAuthorizationObservationIdentity> {
    let ordinal = {
        let update_counter = &NEXT_AUTHORIZATION_OBSERVATION_ORDINAL;
        let mut current = update_counter.load(Ordering::Relaxed);
        loop {
            let Some(updated_value) = current.checked_add(1) else {
                break Err(current);
            };
            match update_counter.compare_exchange_weak(
                current,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => current = reloaded_value,
            }
        }
    }
    .ok()?;
    let snapshot = plan.snapshot();
    let mut bytes = [0_u8; 32];
    bytes[0..8].copy_from_slice(&snapshot.runtime_instance_id.to_le_bytes());
    bytes[8..16].copy_from_slice(&snapshot.snapshot_id.0.to_le_bytes());
    bytes[16..24].copy_from_slice(&snapshot.version_id.as_u64().to_le_bytes());
    bytes[24..32].copy_from_slice(&ordinal.to_le_bytes());
    Some(RelationalAuthorizationObservationIdentity::mint(bytes))
}
