use std::fs;

use worth_foundational::PhysicalIntegrityPosture;
use worth_store_offline_integrity_observer::{
    observe_store, OfflineArtifactDuplicateEvidence, OfflineArtifactObservation,
    OfflineIntegrityOutcome, OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause,
    OfflinePhysicalFormatField, OfflineUnknownPhysicalReason, OfflineUnsupportedVersionAxis,
};

use crate::root_protocol_expected_counters::{assert_counters, expected_counters};
use crate::support::{artifact_path, clean_store, refresh_crc32c, request, Target};

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

#[test]
fn current_selector_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(Target::Current);
}

#[test]
fn previous_selector_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(Target::Previous);
}

#[test]
fn addressed_root_manifest_b_k_l_s_p_t_r_d_u_matrix_is_exact() {
    run_family(Target::Root);
}

fn run_family(target: Target) {
    for operator in [
        Operator::B,
        Operator::K,
        Operator::L,
        Operator::S,
        Operator::P,
        Operator::T,
        Operator::R,
        Operator::D,
        Operator::USchema,
        Operator::UFormat,
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
            u64::from(matches!(operator, Operator::USchema | Operator::UFormat))
        );
        assert_eq!(report.counters().exhausted_bounds(), 0);
    }
}

fn apply_operator(fixture: &crate::support::StoreFixture, target: Target, operator: Operator) {
    let path = artifact_path(fixture, target);
    match operator {
        Operator::R => {
            fs::remove_file(path).unwrap();
        }
        Operator::D => duplicate(fixture, target),
        _ => {
            let mut bytes = fs::read(&path).unwrap();
            match operator {
                Operator::B => bytes[covered_byte(target)] ^= 0x01,
                Operator::K => bytes[44] ^= 0x01,
                Operator::L => {
                    let lied = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) + 1;
                    bytes[24..28].copy_from_slice(&lied.to_le_bytes());
                    refresh_crc32c(&mut bytes);
                }
                Operator::S => {
                    match target {
                        Target::Current | Target::Previous => {
                            bytes[48..64].copy_from_slice(&[
                                0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b,
                                0x2c, 0x2d, 0x2e, 0x2f, 0x30,
                            ]);
                        }
                        Target::Root => {
                            bytes[28..36].copy_from_slice(&2_u64.to_le_bytes());
                            bytes[48..56].copy_from_slice(&2_u64.to_le_bytes());
                        }
                    }
                    refresh_crc32c(&mut bytes);
                }
                Operator::P => {
                    match target {
                        Target::Current | Target::Previous => {
                            bytes[65..73].copy_from_slice(&2_u64.to_le_bytes());
                        }
                        Target::Root => bytes[296..304].fill(0),
                    }
                    refresh_crc32c(&mut bytes);
                }
                Operator::T => {
                    bytes.pop();
                }
                Operator::USchema => {
                    bytes[9] = 3;
                    refresh_crc32c(&mut bytes);
                }
                Operator::UFormat => {
                    bytes[10..12].copy_from_slice(&2_u16.to_le_bytes());
                    if target != Target::Root {
                        bytes[89..91].copy_from_slice(&2_u16.to_le_bytes());
                    }
                    refresh_crc32c(&mut bytes);
                }
                Operator::R | Operator::D => unreachable!(),
            }
            fs::write(path, bytes).unwrap();
        }
    }
}

fn duplicate(fixture: &crate::support::StoreFixture, target: Target) {
    let source = artifact_path(fixture, target);
    let destination = match target {
        Target::Current => fixture
            .records
            .join("root-current-0000000000000001.candidate"),
        Target::Previous => fixture
            .records
            .join("root-previous-0000000000000001.candidate"),
        Target::Root => fixture.roots.join("root-0000000000000002.manifest"),
    };
    fs::hard_link(source, destination).expect("same-volume hard-link fixture must be supported");
}

fn covered_byte(target: Target) -> usize {
    match target {
        Target::Current | Target::Previous => 48,
        Target::Root => 56,
    }
}

fn target_observation(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
) -> &OfflineArtifactObservation {
    let path = match target {
        Target::Current => "families/records/root-current.selector",
        Target::Previous => "families/records/root-previous.selector",
        Target::Root => "families/records/roots/root-0000000000000001.manifest",
    };
    report
        .artifacts()
        .iter()
        .find(|artifact| artifact.relative_path() == path)
        .unwrap()
}

fn assert_localization(artifact: &OfflineArtifactObservation, target: Target, operator: Operator) {
    if target == Target::Root {
        assert_eq!(
            artifact.identity().as_str(),
            "root:0000000000000001",
            "a substituted payload must not replace the parent-addressed artifact identity"
        );
    }
    let artifact_bytes = if target == Target::Root { 368 } else { 107 };
    assert_eq!(
        artifact
            .range()
            .map(|range| (range.offset(), range.length())),
        Some((0, artifact_bytes)),
        "canonical scope must not shrink with acquired bytes: {target:?} {operator:?}"
    );
    match operator {
        Operator::D => assert_eq!(artifact.outcome(), &OfflineIntegrityOutcome::Intact),
        Operator::USchema => assert_unsupported(
            artifact,
            OfflineUnsupportedVersionAxis::EnvelopeSchema,
            3,
            (9, 1),
        ),
        Operator::UFormat => assert_unsupported(
            artifact,
            OfflineUnsupportedVersionAxis::PhysicalRecordFormat,
            2,
            (10, 2),
        ),
        _ => {
            let (cause, range, field, blast) = expected_damage(target, operator, artifact_bytes);
            let OfflineIntegrityOutcome::Damaged(localization) = artifact.outcome() else {
                panic!(
                    "{target:?} {operator:?}: expected damage, got {:?}",
                    artifact.outcome()
                );
            };
            assert_eq!(localization.cause(), cause, "{target:?} {operator:?}");
            assert_eq!(
                localization
                    .damaged_range()
                    .map(|range| (range.offset(), range.length())),
                range,
                "{target:?} {operator:?}"
            );
            assert_eq!(localization.field(), field, "{target:?} {operator:?}");
            assert_eq!(
                localization.blast_radius(),
                blast,
                "{target:?} {operator:?}"
            );
        }
    }
}

fn expected_damage(
    target: Target,
    operator: Operator,
    artifact_bytes: u64,
) -> (
    OfflinePhysicalDamageCause,
    Option<(u64, u64)>,
    Option<OfflinePhysicalFormatField>,
    OfflinePhysicalBlastRadius,
) {
    match operator {
        Operator::B | Operator::K => (
            OfflinePhysicalDamageCause::ChecksumMismatch,
            Some((0, artifact_bytes)),
            None,
            OfflinePhysicalBlastRadius::Frame,
        ),
        Operator::L => (
            OfflinePhysicalDamageCause::Framing,
            Some((24, 4)),
            Some(OfflinePhysicalFormatField::PayloadLength),
            OfflinePhysicalBlastRadius::Field,
        ),
        Operator::S if target == Target::Root => (
            OfflinePhysicalDamageCause::ScopeMismatch,
            Some((48, 8)),
            Some(OfflinePhysicalFormatField::ManifestGeneration),
            OfflinePhysicalBlastRadius::Field,
        ),
        Operator::S => (
            OfflinePhysicalDamageCause::ScopeMismatch,
            Some((48, 16)),
            Some(OfflinePhysicalFormatField::StoreIdentity),
            OfflinePhysicalBlastRadius::Field,
        ),
        Operator::P if target == Target::Root => (
            OfflinePhysicalDamageCause::Pointer,
            Some((296, 8)),
            Some(OfflinePhysicalFormatField::ManifestPointer),
            OfflinePhysicalBlastRadius::ReachableRootSubtree,
        ),
        Operator::P => (
            OfflinePhysicalDamageCause::Pointer,
            Some((65, 8)),
            Some(OfflinePhysicalFormatField::RootGeneration),
            OfflinePhysicalBlastRadius::ReachableRootSubtree,
        ),
        Operator::T => (
            OfflinePhysicalDamageCause::Truncation,
            Some((artifact_bytes - 1, 1)),
            None,
            OfflinePhysicalBlastRadius::Artifact,
        ),
        Operator::R => (
            OfflinePhysicalDamageCause::MissingArtifact,
            None,
            None,
            if target == Target::Root {
                OfflinePhysicalBlastRadius::ReachableRootSubtree
            } else {
                OfflinePhysicalBlastRadius::Artifact
            },
        ),
        Operator::D => unreachable!(),
        Operator::USchema | Operator::UFormat => unreachable!(),
    }
}

fn assert_related_observations(
    report: &worth_store_offline_integrity_observer::OfflineIntegrityReport,
    target: Target,
    operator: Operator,
) {
    if matches!(operator, Operator::D) {
        let duplicates: Vec<_> = report
            .artifacts()
            .iter()
            .filter(|artifact| !artifact.duplicates().is_empty())
            .collect();
        assert_eq!(
            duplicates.len(),
            1,
            "{target:?} D: {:?}",
            report.artifacts()
        );
        assert_eq!(
            duplicates[0].outcome(),
            &OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected
            )
        );
        assert!(matches!(
            duplicates[0].duplicates(),
            [OfflineArtifactDuplicateEvidence::PhysicalAlias { .. }]
        ));
    }
    if matches!(target, Target::Current | Target::Previous) && matches!(operator, Operator::P) {
        let missing = report
            .artifacts()
            .iter()
            .find(|artifact| {
                artifact.relative_path() == "families/records/roots/root-0000000000000002.manifest"
            })
            .expect("selector P reports its missing addressed root");
        let OfflineIntegrityOutcome::Damaged(localization) = missing.outcome() else {
            panic!("missing addressed root must be damaged");
        };
        assert_eq!(
            localization.cause(),
            OfflinePhysicalDamageCause::MissingArtifact
        );
        assert_eq!(
            localization.blast_radius(),
            OfflinePhysicalBlastRadius::ReachableRootSubtree
        );
    }
}

fn assert_unsupported(
    artifact: &OfflineArtifactObservation,
    axis: OfflineUnsupportedVersionAxis,
    observed: u64,
    range: (u64, u64),
) {
    assert_eq!(
        artifact.outcome().posture(),
        PhysicalIntegrityPosture::Unsupported
    );
    let OfflineIntegrityOutcome::Unsupported(value) = artifact.outcome() else {
        unreachable!()
    };
    assert_eq!(value.axis(), axis);
    assert_eq!(value.observed(), observed);
    assert_eq!((value.range().offset(), value.range().length()), range);
}
