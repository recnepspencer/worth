use super::clean_artifact_manifest::RootArtifactManifestDeclaration;
use super::frame_checksum::refresh_checksum;
use super::oracle_expectation_assertions::assert_exact_parent_expectation;
use super::test_world::{
    advance_expected_counters, assert_clean_baseline_unchanged, fresh_row, root_fixture,
    tree_statistics,
};
use super::*;

const ROLES: [RootArtifactRole; 3] = [
    RootArtifactRole::CurrentSelector,
    RootArtifactRole::PreviousSelector,
    RootArtifactRole::AddressedRootManifest,
];
const CODES: [RootCorruptionCode; 9] = [
    RootCorruptionCode::B,
    RootCorruptionCode::K,
    RootCorruptionCode::L,
    RootCorruptionCode::S,
    RootCorruptionCode::P,
    RootCorruptionCode::T,
    RootCorruptionCode::R,
    RootCorruptionCode::D,
    RootCorruptionCode::U,
];

#[test]
fn clean_manifest_records_exact_root_roles() {
    let fixture = root_fixture();
    assert_ne!(fixture.manifest.identity(), [0; 32]);
    assert_eq!(fixture.manifest.records().count(), 3);
    for role in ROLES {
        let identity = fixture.manifest.target_for_role(role);
        let record = fixture.manifest.record(identity).unwrap();
        assert_eq!(record.identity(), identity);
        assert_eq!(identity.role(), role);
        assert_ne!(identity.store_identity(), [0; 16]);
        assert_ne!(identity.concrete_identity(), 0);
        assert_ne!(identity.root_generation(), 0);
        assert_eq!(
            record.exact_length(),
            if role == RootArtifactRole::AddressedRootManifest {
                368
            } else {
                107
            }
        );
        assert_eq!(record.covered_ranges(), &[0..44, 48..record.exact_length()]);
        assert_eq!(record.checksum_range(), 44..48);
        assert_eq!(record.length_range(), 24..28);
        assert_eq!(record.version_range(), 10..12);
        assert_eq!(
            record.expected_reachable_paths().len(),
            if role == RootArtifactRole::AddressedRootManifest {
                1
            } else {
                2
            }
        );
    }
    let mut counters = RootLocalizationCounters::default();
    assert!(matches!(
        FreshRootArtifactRow::copy_from(
            &fixture.scenario,
            &fixture.manifest,
            fixture.root.path().join("reports"),
            &mut counters,
        ),
        Err(FreshRootArtifactRowDenial::ReportPathCollision)
    ));
}

#[test]
fn clean_manifest_rejects_checksum_valid_same_scope_substitution_source() {
    let fixture = root_fixture();
    let target = fixture
        .manifest
        .target_for_role(RootArtifactRole::CurrentSelector);
    let record = fixture.manifest.record(target).unwrap();
    let same_scope_path = std::path::PathBuf::from("substitution-sources/same-scope.selector");
    let mut same_scope_bytes = std::fs::read(
        fixture
            .scenario
            .clean_store_root()
            .join(record.relative_path()),
    )
    .unwrap();
    same_scope_bytes[90] ^= 1;
    refresh_checksum(&mut same_scope_bytes);
    std::fs::write(
        fixture.scenario.clean_store_root().join(&same_scope_path),
        same_scope_bytes,
    )
    .unwrap();
    let declarations = fixture
        .manifest
        .records()
        .map(|record| RootArtifactManifestDeclaration {
            identity: record.identity(),
            relative_path: record.relative_path().to_path_buf(),
            substitution_source_path: if record.identity() == target {
                same_scope_path.clone()
            } else {
                record.substitution_source_path().to_path_buf()
            },
            substitution_source_identity: if record.identity() == target {
                target
            } else {
                record.substitution_source_identity()
            },
            duplicate_path: record.duplicate_path().to_path_buf(),
            covered_edit_offset: record.covered_edit_offset(),
            pointer_range: record.pointer_range(),
            expected_reachable_paths: record.expected_reachable_paths().to_vec(),
        })
        .collect();
    let supporting = fixture
        .manifest
        .supporting_artifacts()
        .map(|(path, _)| path.to_path_buf())
        .collect();
    assert_eq!(
        CleanRootArtifactManifest::observe(
            fixture.scenario.clean_store_root(),
            declarations,
            supporting,
        ),
        Err(RootArtifactManifestDenial::InvalidSubstitutionScope(target))
    );
}

#[test]
fn every_root_operator_uses_a_fresh_copy_and_passes_exact_editor_oracle_audit() {
    let fixture = root_fixture();
    let mut counters = RootLocalizationCounters::default();
    let mut expected_counters = RootLocalizationCounters::default();
    let (clean_files, clean_bytes) = tree_statistics(fixture.scenario.clean_store_root());
    let mut row_number = 0_u64;
    for role in ROLES {
        let target = fixture.manifest.target_for_role(role);
        for code in CODES {
            let edit = DeclaredRootCorruption::for_code(&fixture.manifest, target, code).unwrap();
            let expected =
                derive_parent_expectation(&fixture.manifest, &edit, &mut counters).unwrap();
            row_number += 1;
            let row = FreshRootArtifactRow::copy_from(
                &fixture.scenario,
                &fixture.manifest,
                fixture.root.path().join(format!("rows/row-{row_number}")),
                &mut counters,
            )
            .unwrap();
            assert_eq!(row.baseline_identity(), fixture.manifest.identity());
            let audit =
                apply_declared_corruption(row.root(), &fixture.manifest, &edit, &mut counters)
                    .unwrap();
            let (after_files, after_bytes) = tree_statistics(row.root());
            advance_expected_counters(
                &mut expected_counters,
                code,
                fixture.manifest.record(target).unwrap().exact_length(),
                clean_files,
                clean_bytes,
                after_files,
                after_bytes,
            );
            assert_eq!(counters, expected_counters);
            assert_eq!(audit.target(), target);
            assert_eq!(audit.declaration_identity(), edit.identity());
            assert_eq!(expected.target(), target);
            assert_eq!(expected.manifest_identity(), fixture.manifest.identity());
            assert_eq!(expected.edit_identity(), edit.identity());
            assert_operator_contract(&fixture.manifest, &edit, &audit, &expected);
        }
    }
    assert_eq!(counters, expected_counters);
    assert_clean_baseline_unchanged(&fixture);
}

#[test]
fn editor_rejects_changed_target_and_changed_substitution_source() {
    let fixture = root_fixture();
    let target = fixture
        .manifest
        .target_for_role(RootArtifactRole::CurrentSelector);
    let mut counters = RootLocalizationCounters::default();

    let changed_target = fresh_row(&fixture, "changed-target", &mut counters);
    let record = fixture.manifest.record(target).unwrap();
    let target_path = changed_target.root().join(record.relative_path());
    let mut bytes = std::fs::read(&target_path).unwrap();
    bytes[65] ^= 1;
    std::fs::write(target_path, bytes).unwrap();
    let edit =
        DeclaredRootCorruption::for_code(&fixture.manifest, target, RootCorruptionCode::B).unwrap();
    assert_eq!(
        apply_declared_corruption(
            changed_target.root(),
            &fixture.manifest,
            &edit,
            &mut counters
        ),
        Err(EditorAuditDenial::BaselineChanged)
    );

    let changed_source = fresh_row(&fixture, "changed-source", &mut counters);
    let source_path = changed_source
        .root()
        .join(record.substitution_source_path());
    let mut donor = std::fs::read(&source_path).unwrap();
    donor[65] ^= 1;
    std::fs::write(source_path, donor).unwrap();
    let edit =
        DeclaredRootCorruption::for_code(&fixture.manifest, target, RootCorruptionCode::S).unwrap();
    assert_eq!(
        apply_declared_corruption(
            changed_source.root(),
            &fixture.manifest,
            &edit,
            &mut counters
        ),
        Err(EditorAuditDenial::SourceChanged)
    );
}

fn assert_operator_contract(
    manifest: &CleanRootArtifactManifest,
    edit: &DeclaredRootCorruption,
    audit: &EditorResultAudit,
    expected: &ExpectedRootLocalization,
) {
    let record = manifest.record(edit.target()).unwrap();
    assert_exact_parent_expectation(record, edit, expected);
    assert_eq!(audit.before_sha256(), record.content_sha256());
    match edit.code() {
        RootCorruptionCode::R => assert_eq!(audit.after_sha256(), None),
        RootCorruptionCode::S => assert_eq!(
            audit.after_sha256(),
            Some(record.substitution_source_sha256())
        ),
        RootCorruptionCode::D => assert_eq!(audit.after_sha256(), Some(record.content_sha256())),
        _ => assert_ne!(audit.after_sha256(), Some(record.content_sha256())),
    }
    match edit.code() {
        RootCorruptionCode::B => {
            let changed = record.covered_edit_offset()..record.covered_edit_offset() + 1;
            assert_eq!(audit.changed_ranges(), std::slice::from_ref(&changed));
            assert_eq!(audit.checksum_valid_after_edit(), Some(false));
            assert_eq!(
                expected.cause(),
                ExpectedRootCause::CoveredByteIntegrityMismatch
            );
        }
        RootCorruptionCode::K => {
            let changed = record.checksum_range().start..record.checksum_range().start + 1;
            assert_eq!(audit.changed_ranges(), std::slice::from_ref(&changed));
            assert_eq!(audit.checksum_valid_after_edit(), Some(false));
            assert_eq!(expected.cause(), ExpectedRootCause::ChecksumFieldDamage);
        }
        RootCorruptionCode::L => {
            assert_eq!(audit.checksum_valid_after_edit(), Some(true));
            assert_eq!(expected.expected_ranges(), &[record.length_range()]);
        }
        RootCorruptionCode::S => {
            assert_eq!(audit.checksum_valid_after_edit(), Some(true));
            assert_eq!(audit.changed_ranges(), record.substitution_changed_ranges());
            assert_eq!(expected.posture(), ExpectedRootPosture::Damaged);
        }
        RootCorruptionCode::P => {
            assert_eq!(audit.checksum_valid_after_edit(), Some(true));
            assert_eq!(expected.expected_ranges(), &[record.pointer_range()]);
            assert_eq!(
                expected.minimum_reachable_paths().collect::<Vec<_>>(),
                record
                    .expected_reachable_paths()
                    .iter()
                    .map(std::path::PathBuf::as_path)
                    .collect::<Vec<_>>()
            );
        }
        RootCorruptionCode::T => {
            assert_eq!(audit.checksum_valid_after_edit(), Some(false));
            assert_eq!(expected.cause(), ExpectedRootCause::Truncated);
        }
        RootCorruptionCode::R => {
            assert_eq!(audit.removed_path(), Some(record.relative_path()));
            assert_eq!(audit.checksum_valid_after_edit(), None);
            assert_eq!(expected.posture(), ExpectedRootPosture::Missing);
            assert_eq!(
                expected.minimum_reachable_paths().count(),
                record.expected_reachable_paths().len()
            );
        }
        RootCorruptionCode::D => {
            assert_eq!(audit.created_path(), Some(record.duplicate_path()));
            assert_eq!(audit.checksum_valid_after_edit(), Some(true));
            assert_eq!(expected.posture(), ExpectedRootPosture::Duplicate);
        }
        RootCorruptionCode::U => {
            assert_eq!(audit.checksum_valid_after_edit(), Some(true));
            assert_eq!(expected.posture(), ExpectedRootPosture::Unsupported);
            assert_eq!(
                expected.minimum_blast_radius(),
                ExpectedMinimumBlastRadius::NoDamageClaim
            );
        }
    }
    assert_ne!(expected.expected_ranges(), &[]);
    if !matches!(edit.code(), RootCorruptionCode::P | RootCorruptionCode::R) {
        assert_eq!(expected.minimum_reachable_paths().count(), 0);
    }
    let _ = expected.minimum_blast_radius();
}
