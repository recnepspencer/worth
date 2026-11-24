//! Reopens the replay-pin fixture from a selected external checkpoint.

use super::ColdPinSource;

pub(super) fn cold_pin_recovery(
    native: crate::durability::data::RelationalNativeCheckpoint,
) -> (
    crate::durability::published_medium::RelationalDurableRecoverySource,
    std::sync::Arc<ColdPinSource>,
) {
    let members = native
        .members()
        .iter()
        .map(|m| match m {
            crate::durability::data::RelationalCheckpointMember::New { locator, bytes } => {
                (*locator, bytes.clone())
            }
            crate::durability::data::RelationalCheckpointMember::Carried { .. } => unreachable!(),
        })
        .collect();
    let source = std::sync::Arc::new(ColdPinSource {
        members,
        damaged: std::sync::atomic::AtomicBool::new(false),
        reads: Default::default(),
    });
    let recovery = crate::durability::published_medium::RelationalDurableRecoverySource::new(
        Some(
            crate::durability::published_medium::RelationalDurableCheckpointSource::new(
                native.covered_position().unwrap(),
                native.covered_branch_transitions().unwrap(),
                native,
            ),
        ),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    (recovery, source)
}
