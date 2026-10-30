use super::localization_expectations::{assert_localization, assert_related_observations};
use crate::support::checksum::refresh_crc32c;
pub(crate) use std::fs;

pub(crate) use worth_foundational::PhysicalIntegrityPosture;
pub(crate) use worth_store_offline_integrity_observer::{
    observe_store, OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIntegrityOutcome, OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause,
    OfflinePhysicalFormatField, OfflineUnknownPhysicalReason, OfflineUnsupportedVersionAxis,
};

pub(crate) use crate::root_protocol_expected_counters::{assert_counters, expected_counters};
pub(crate) use crate::{
    store_offline_integrity_observer_support::artifact_path,
    store_offline_integrity_observer_support::request,
    store_offline_integrity_observer_support::Target, support::fixtures::clean_store,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Operator {
    B,
    K,
    L,
    S,
    P,
    T,
    R,
    D,
    USchema,
    UFormat,
}

pub(crate) fn run_family(target: Target) {
    for operator in [
        crate::root_protocol_corruption_matrix::Operator::B,
        crate::root_protocol_corruption_matrix::Operator::K,
        crate::root_protocol_corruption_matrix::Operator::L,
        crate::root_protocol_corruption_matrix::Operator::S,
        crate::root_protocol_corruption_matrix::Operator::P,
        crate::root_protocol_corruption_matrix::Operator::T,
        crate::root_protocol_corruption_matrix::Operator::R,
        crate::root_protocol_corruption_matrix::Operator::D,
        crate::root_protocol_corruption_matrix::Operator::USchema,
        crate::root_protocol_corruption_matrix::Operator::UFormat,
    ] {
        let fixture = clean_store(&format!("matrix-{target:?}-{operator:?}"));
        apply_operator(&fixture, target, operator);
        let report = observe_store(&request(&fixture))
            .unwrap_or_else(|denial| panic!("{target:?} {operator:?}: {denial:?}"));
        let artifact = target_observation(&report, target);
        assert_localization(artifact, target, operator);
        assert_related_observations(&report, target, operator);
        assert_counters(report.counters(), expected_counters(target, operator));
        assert_eq!(
            report.counters().unsupported_versions(),
            u64::from(matches!(
                operator,
                crate::root_protocol_corruption_matrix::Operator::USchema
                    | crate::root_protocol_corruption_matrix::Operator::UFormat
            ))
        );
        assert_eq!(report.counters().exhausted_bounds(), 0);
    }
}

pub(crate) fn apply_operator(
    fixture: &crate::support::fixtures::StoreFixture,
    target: Target,
    operator: Operator,
) {
    let path = artifact_path(fixture, target);
    match operator {
        crate::root_protocol_corruption_matrix::Operator::R => {
            std::fs::remove_file(path).unwrap();
        }
        crate::root_protocol_corruption_matrix::Operator::D => duplicate(fixture, target),
        _ => {
            let mut bytes = std::fs::read(&path).unwrap();
            match operator {
                crate::root_protocol_corruption_matrix::Operator::B => {
                    bytes[covered_byte(target)] ^= 0x01
                }
                crate::root_protocol_corruption_matrix::Operator::K => bytes[44] ^= 0x01,
                crate::root_protocol_corruption_matrix::Operator::L => {
                    let lied = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) + 1;
                    bytes[24..28].copy_from_slice(&lied.to_le_bytes());
                    refresh_crc32c(&mut bytes);
                }
                crate::root_protocol_corruption_matrix::Operator::S => {
                    match target {
                        crate::store_offline_integrity_observer_support::Target::Current
                        | crate::store_offline_integrity_observer_support::Target::Previous => {
                            bytes[48..64].copy_from_slice(&[
                                0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b,
                                0x2c, 0x2d, 0x2e, 0x2f, 0x30,
                            ]);
                        }
                        crate::store_offline_integrity_observer_support::Target::Root => {
                            bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
                            bytes[48..56].copy_from_slice(&2_u64.to_le_bytes());
                        }
                    }
                    refresh_crc32c(&mut bytes);
                }
                crate::root_protocol_corruption_matrix::Operator::P => {
                    match target {
                        crate::store_offline_integrity_observer_support::Target::Current
                        | crate::store_offline_integrity_observer_support::Target::Previous => {
                            bytes[65..73].copy_from_slice(&2_u64.to_le_bytes());
                        }
                        crate::store_offline_integrity_observer_support::Target::Root => {
                            bytes[296..304].fill(0)
                        }
                    }
                    refresh_crc32c(&mut bytes);
                }
                crate::root_protocol_corruption_matrix::Operator::T => {
                    bytes.pop();
                }
                crate::root_protocol_corruption_matrix::Operator::USchema => {
                    bytes[9] = if target
                        == crate::store_offline_integrity_observer_support::Target::Root
                    {
                        8
                    } else {
                        3
                    };
                    refresh_crc32c(&mut bytes);
                }
                crate::root_protocol_corruption_matrix::Operator::UFormat => {
                    bytes[10..12].copy_from_slice(&3_u16.to_le_bytes());
                    if target != crate::store_offline_integrity_observer_support::Target::Root {
                        bytes[89..91].copy_from_slice(&3_u16.to_le_bytes());
                    }
                    refresh_crc32c(&mut bytes);
                }
                crate::root_protocol_corruption_matrix::Operator::R
                | crate::root_protocol_corruption_matrix::Operator::D => unreachable!(),
            }
            std::fs::write(path, bytes).unwrap();
        }
    }
}

pub(crate) fn duplicate(fixture: &crate::support::fixtures::StoreFixture, target: Target) {
    let source = artifact_path(fixture, target);
    let destination = match target {
        crate::store_offline_integrity_observer_support::Target::Current => fixture
            .records
            .join("root-current-0000000000000001.candidate"),
        crate::store_offline_integrity_observer_support::Target::Previous => fixture
            .records
            .join("root-previous-0000000000000001.candidate"),
        crate::store_offline_integrity_observer_support::Target::Root => {
            fixture.roots.join("root-0000000000000002.manifest")
        }
    };
    std::fs::hard_link(source, destination)
        .expect("same-volume hard-link fixture must be supported");
}

pub(crate) fn covered_byte(target: Target) -> usize {
    match target {
        crate::store_offline_integrity_observer_support::Target::Current
        | crate::store_offline_integrity_observer_support::Target::Previous => 48,
        crate::store_offline_integrity_observer_support::Target::Root => 56,
    }
}

pub(crate) fn target_observation(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
) -> &OfflineArtifactObservation {
    let path = match target {
        crate::store_offline_integrity_observer_support::Target::Current => {
            "families/records/root-current.selector"
        }
        crate::store_offline_integrity_observer_support::Target::Previous => {
            "families/records/root-previous.selector"
        }
        crate::store_offline_integrity_observer_support::Target::Root => {
            "families/records/roots/root-0000000000000001.manifest"
        }
    };
    report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == path)
        .unwrap()
}
