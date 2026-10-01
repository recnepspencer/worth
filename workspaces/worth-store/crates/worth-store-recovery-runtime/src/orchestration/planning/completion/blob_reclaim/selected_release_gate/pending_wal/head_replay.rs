//! Admit the exact C.9 effect against actual selected head-path media.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, PhysicalRedoProjection, PhysicalSourceSelection,
    VerifiedSelectedReleaseHeadReplayV14,
};

use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    projection: &PhysicalRedoProjection,
    redo: &ImmutablePhysicalRedoPlan,
    selected: &PhysicalSourceSelection,
    staging_bytes: u64,
    resident: &mut ResidentAllowance,
) -> Option<VerifiedSelectedReleaseHeadReplayV14> {
    let worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.materialization().operation()
    else {
        return None;
    };
    let format = selected.root().selected().selector().format();
    let peak = effect
        .verification_additional_peak_bytes(format)?
        .max(u64::from(format.page_size().bytes()).checked_mul(2)?)
        .max(effect.owned_heap_bytes()?);
    resident.transient(peak).ok()?;
    let replay = VerifiedSelectedReleaseHeadReplayV14::admit_projection(
        projection,
        redo,
        selected,
        staging_bytes,
        resident.remaining(),
        |reference, maximum| {
            budget.consume(1).map_err(|_| ())?;
            discovery
                .read_release_custody_head_block(reference.generation(), reference.block(), maximum)
                .map_err(|_| ())?
                .bytes()
                .map(<[u8]>::to_vec)
                .ok_or(())
        },
    )
    .ok()?;
    resident.bytes(replay.owned_heap_bytes()?).ok()?;
    Some(replay)
}
