use worth_store::physical_runtime::{ExternalPhysicalRecordLocator, PhysicalRecordId};
use worth_store_offline_verifier::OfflineDurableManifestWalk;

use super::super::scale_invalid_worlds::InvalidScaleWorlds;
use super::super::scenario_evidence::ScenarioProcessEvidence;
use super::ScaleObservation;

pub(super) struct ScaleCourtroomEvidence<'world> {
    pub(super) record_count: u16,
    pub(super) last: PhysicalRecordId,
    pub(super) locator: ExternalPhysicalRecordLocator,
    pub(super) walk: &'world OfflineDurableManifestWalk,
    pub(super) processes: &'world [ScenarioProcessEvidence],
    pub(super) observation: ScaleObservation,
    pub(super) invalid: &'world InvalidScaleWorlds,
}

pub(super) fn emit(input: ScaleCourtroomEvidence<'_>) {
    assert_eq!(
        input.last.ordinal(),
        locator_record_ordinal(input.locator),
        "locator must be readmitted"
    );
    assert_eq!(
        usize::from(input.record_count),
        input.walk.placements().len(),
        "runtime and offline record counts must agree"
    );
    assert_eq!(
        u64::from(input.record_count),
        input.observation.scan_records,
        "scan must visit every record"
    );
    assert!(input.observation.point_blocks <= input.observation.whole_blocks);
    assert_eq!(input.observation.point_allocations, 16_384);
    assert!(input.observation.scan_allocations >= input.observation.point_allocations);
    assert!(input.observation.scan_allocations < 65_536);
    assert_eq!(input.observation.signal_invalidation_delta, 0);
    assert_eq!(input.observation.invalid_worlds, 5);
    assert!(input.invalid.missing_catalog_refused);
    assert!(input.invalid.checksum_damage_refused);
    assert!(input.invalid.stale_manifest_refused);
    assert!(input.invalid.format_drift_refused);
    assert!(input.invalid.residue_excluded);
    super::super::scenario_evidence::assert_distinct_processes(input.processes);
}

fn locator_record_ordinal(locator: ExternalPhysicalRecordLocator) -> u64 {
    u64::from_le_bytes(locator.encode()[32..40].try_into().unwrap())
}
