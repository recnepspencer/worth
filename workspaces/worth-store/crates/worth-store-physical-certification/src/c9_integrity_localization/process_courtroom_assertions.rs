use serde_json::Value;
use sha2::{Digest, Sha256};

use super::process_recovery_observation::{
    ProcessBlastRadius, ProcessByteRange, ProcessDamageCause, ProcessIntegrityArtifactFamily,
    ProcessIntegrityRejection, ProcessRecoveryBlockCause, ProcessRecoveryObservation,
    ProcessRecoveryPosture, ProcessRootProtocolArtifact, ProcessRootProtocolDenialKind,
};
use super::{ClosedStoreProcessManifest, DeclaredProcessPoison, ProcessRootCase, RootArtifactRole};

pub(super) fn assert_addressed_root_poison_preserves_current_selector(
    row: &std::path::Path,
    manifest: &ClosedStoreProcessManifest,
) {
    let record = manifest
        .artifact(RootArtifactRole::CurrentSelector)
        .expect("current selector record");
    let bytes = std::fs::read(row.join(record.relative_path())).expect("read current selector");
    assert_eq!(Sha256::digest(&bytes).as_slice(), record.content_sha256());
    assert_eq!(bytes[64], 1, "current role remains unchanged");
    assert_eq!(
        u64::from_le_bytes(bytes[28..36].try_into().unwrap()),
        record.concrete_identity()
    );
    assert_eq!(
        u64::from_le_bytes(bytes[65..73].try_into().unwrap()),
        record.root_generation()
    );
    assert_eq!(&bytes[48..64], manifest.store_identity());
}

pub(super) fn assert_recovery_expectation(
    observation: &ProcessRecoveryObservation,
    manifest: &ClosedStoreProcessManifest,
    case: ProcessRootCase,
) {
    assert_eq!(
        observation.observed_store_identity,
        Some(manifest.store_identity()),
        "recovery reports the Store identity observed by its real outcome"
    );
    match case {
        ProcessRootCase::CleanControl => assert_clean_recovery(observation),
        ProcessRootCase::PoisonCurrentSelector => {
            let counters = observation.discovery.expect("blocked discovery counters");
            assert_eq!(observation.posture, ProcessRecoveryPosture::Recovered);
            assert!(
                observation.recovery_effects > 0,
                "fallback actually replays the admitted publication"
            );
            assert_eq!(counters.current_selector_integrity_admissions, 0);
            assert_eq!(counters.current_selector_interpretations, 0);
            assert_eq!(counters.current_root_integrity_admissions, 0);
            assert_eq!(counters.current_root_candidate_interpretations, 0);
            assert_eq!(counters.previous_selector_integrity_admissions, 1);
            assert_eq!(counters.previous_selector_interpretations, 1);
            assert_eq!(counters.previous_root_integrity_admissions, 1);
            assert_eq!(counters.previous_root_candidate_interpretations, 1);
            assert_eq!(
                observation
                    .root_protocol
                    .successor_root_integrity_admissions,
                1
            );
            assert_eq!(observation.root_protocol.successor_root_interpretations, 1);
            assert_eq!(
                observation
                    .root_protocol
                    .staged_selector_integrity_admissions,
                1
            );
            assert_eq!(
                observation.root_protocol.closeout_selector_interpretations,
                1
            );
            assert_exact_recovery_damage(
                observation,
                manifest,
                RootArtifactRole::CurrentSelector,
                ProcessRootProtocolArtifact::CurrentSelector,
                ProcessIntegrityArtifactFamily::CurrentRootSelector,
                None,
            );
        }
        ProcessRootCase::PoisonAddressedRoot => {
            let counters = observation.discovery.expect("blocked discovery counters");
            let root = manifest
                .artifact(RootArtifactRole::AddressedRootManifest)
                .expect("addressed root manifest");
            assert_eq!(
                observation.posture,
                ProcessRecoveryPosture::Blocked(ProcessRecoveryBlockCause::RootProtocol)
            );
            assert_eq!(observation.recovery_effects, 0);
            assert_eq!(counters.current_selector_integrity_admissions, 1);
            assert_eq!(counters.current_selector_interpretations, 1);
            assert_eq!(counters.current_root_integrity_admissions, 0);
            assert_eq!(counters.current_root_candidate_interpretations, 0);
            assert_eq!(observation.root_protocol, Default::default());
            assert_exact_recovery_damage(
                observation,
                manifest,
                RootArtifactRole::AddressedRootManifest,
                ProcessRootProtocolArtifact::CurrentRoot {
                    generation: root.root_generation(),
                },
                ProcessIntegrityArtifactFamily::RootManifest,
                Some(root.root_generation()),
            );
        }
    }
}

fn assert_clean_recovery(observation: &ProcessRecoveryObservation) {
    let counters = observation.discovery.expect("recovered discovery counters");
    assert_eq!(observation.posture, ProcessRecoveryPosture::Recovered);
    assert_eq!(observation.recovery_effects, 0);
    assert_eq!(counters.current_selector_integrity_admissions, 1);
    assert_eq!(counters.current_selector_interpretations, 1);
    assert_eq!(counters.current_root_integrity_admissions, 1);
    assert_eq!(counters.current_root_candidate_interpretations, 1);
    assert_eq!(observation.root_protocol, Default::default());
}

fn assert_exact_recovery_damage(
    observation: &ProcessRecoveryObservation,
    manifest: &ClosedStoreProcessManifest,
    role: RootArtifactRole,
    expected_artifact: ProcessRootProtocolArtifact,
    expected_family: ProcessIntegrityArtifactFamily,
    expected_generation: Option<u64>,
) {
    let record = manifest.artifact(role).expect("damaged process artifact");
    let matching = observation
        .root_protocol_denials
        .iter()
        .filter(|denial| denial.artifact == expected_artifact)
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "exact runtime root denial");
    let ProcessRootProtocolDenialKind::Integrity(ProcessIntegrityRejection::Damaged(damage)) =
        matching[0].denial
    else {
        panic!("runtime root denial did not retain damaged integrity localization");
    };
    assert_eq!(damage.scope.store_identity, manifest.store_identity());
    assert_eq!(damage.scope.family, expected_family);
    assert_eq!(damage.scope.root_generation, expected_generation);
    assert_eq!(
        damage.scope.byte_range,
        ProcessByteRange {
            offset: 0,
            length: record.exact_length(),
        }
    );
    assert!(damage
        .scope
        .record_format_identity
        .is_some_and(|identity| identity != [0; 10]));
    assert_eq!(damage.cause, ProcessDamageCause::ChecksumMismatch);
    assert_eq!(
        damage.damaged_range,
        ProcessByteRange {
            offset: 0,
            length: record.exact_length(),
        }
    );
    assert_eq!(damage.field, None);
    assert_eq!(damage.blast_radius, ProcessBlastRadius::CanonicalFrame);
}

pub(super) fn assert_offline_expectation(
    report: &Value,
    manifest: &ClosedStoreProcessManifest,
    poison: Option<&DeclaredProcessPoison>,
) {
    for role in [
        RootArtifactRole::CurrentSelector,
        RootArtifactRole::AddressedRootManifest,
    ] {
        let artifact = manifest.artifact(role).expect("decisive artifact");
        assert_eq!(artifact.role(), role);
        assert_ne!(artifact.concrete_identity(), 0);
        let path = artifact
            .relative_path()
            .to_string_lossy()
            .replace('\\', "/");
        let observed = report["artifacts"]
            .as_array()
            .expect("artifact array")
            .iter()
            .find(|entry| entry["path"] == path)
            .expect("manifest artifact appears in offline report");
        if poison.map(DeclaredProcessPoison::role) == Some(role) {
            assert_eq!(observed["outcome"]["posture"], "damaged");
            assert_eq!(observed["outcome"]["cause"], "checksum_mismatch");
            assert!(observed["outcome"]["field"].is_null());
            assert_eq!(observed["outcome"]["blast_radius"], "frame");
            assert_eq!(observed["outcome"]["damaged_range"]["offset"], 0);
            assert_eq!(
                observed["outcome"]["damaged_range"]["length"],
                artifact.exact_length()
            );
        } else if poison.is_none() {
            assert_eq!(observed["outcome"]["posture"], "intact");
        } else if poison.map(DeclaredProcessPoison::role) == Some(RootArtifactRole::CurrentSelector)
            && role == RootArtifactRole::AddressedRootManifest
        {
            assert_eq!(observed["outcome"]["posture"], "unknown");
            assert_eq!(observed["outcome"]["reason"], "root_not_addressed");
        } else {
            assert_eq!(observed["outcome"]["posture"], "intact");
            assert_eq!(
                observed["duplicates"].as_array().map(Vec::len),
                Some(0),
                "non-target root-protocol artifact remains canonical and non-duplicated"
            );
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct DecoderCounters {
    pub(super) checksum_calculations: u64,
    pub(super) checksum_validated_frames: u64,
    pub(super) selector_payload_entries: u64,
    pub(super) root_manifest_payload_entries: u64,
}

impl DecoderCounters {
    pub(super) fn from_report(report: &Value) -> Self {
        Self {
            checksum_calculations: report["consumed"]["checksum_calculations"]
                .as_u64()
                .expect("checksum calculation counter"),
            checksum_validated_frames: report["consumed"]["durable_frame_decoders"]
                .as_u64()
                .expect("checksum-validated frame counter"),
            selector_payload_entries: report["consumed"]["selector_decoders"]
                .as_u64()
                .expect("selector payload decoder-entry counter"),
            root_manifest_payload_entries: report["consumed"]["root_manifest_decoders"]
                .as_u64()
                .expect("root-manifest payload decoder-entry counter"),
        }
    }
}
