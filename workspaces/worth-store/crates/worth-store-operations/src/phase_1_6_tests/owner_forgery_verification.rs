use super::support::*;
use sha2::{Digest, Sha256};
use worth_store_offline_verifier::BackupArtifactSemanticDefectKind;
use worth_store_physical_format::{
    BackupBundleArtifactFamily, BackupBundleArtifactManifestRow, BackupBundleManifestDeclaration,
    MaterializedBackupBundle, PHYSICAL_HEADER_LENGTH,
};

const OWNER_SECONDARY_START: usize = 26;
const OWNER_SECONDARY_END: usize = 34;

#[test]
fn self_consistent_outer_hashes_cannot_launder_invalid_owner_artifacts() {
    let cases = [
        BackupBundleArtifactFamily::RootManifest,
        BackupBundleArtifactFamily::CheckpointManifest,
        BackupBundleArtifactFamily::WalSegment,
        BackupBundleArtifactFamily::Page,
        BackupBundleArtifactFamily::Extent,
        BackupBundleArtifactFamily::Index,
        BackupBundleArtifactFamily::BlobChunk,
    ];
    for family in cases {
        let (_scenario, bundle) = materialized_bundle(&format!("owner-forgery-{family:?}"));
        rewrite_artifact_and_manifest(&bundle, family, |bytes| forge_owner_bytes(family, bytes));
        assert_owner_defect(bundle.root(), family, None);
    }
}

#[test]
fn same_generation_cross_owner_substitution_survives_outer_hashes_but_not_owner_decode() {
    for family in [
        BackupBundleArtifactFamily::Page,
        BackupBundleArtifactFamily::Extent,
    ] {
        let (_scenario, bundle) = materialized_bundle(&format!("cross-owner-{family:?}"));
        rewrite_artifact_and_manifest(&bundle, family, |bytes| {
            assert!(bytes.len() >= usize::from(PHYSICAL_HEADER_LENGTH));
            bytes[OWNER_SECONDARY_START..OWNER_SECONDARY_END]
                .copy_from_slice(&999_u64.to_le_bytes());
        });
        assert_owner_defect(
            bundle.root(),
            family,
            Some(BackupArtifactSemanticDefectKind::OwnerBindingMismatch),
        );
    }
}

fn materialized_bundle(case: &str) -> (BackupScenario, MaterializedBackupBundle) {
    let scenario = BackupScenario::new(case);
    let authority = scenario.authority();
    let control = scenario.control_store();
    let bundle = OnlineBackupIntent::new(
        OperationalOperationId::new(format!("backup-{case}")).expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut")
    .materialize(&scenario.target, 31, &control)
    .expect("session")
    .finish()
    .expect("materialize")
    .into_parts()
    .0;
    (scenario, bundle)
}

fn assert_owner_defect(
    root: &std::path::Path,
    family: BackupBundleArtifactFamily,
    expected_kind: Option<BackupArtifactSemanticDefectKind>,
) {
    let reopened = BackupBundleFormatAuthority::canonical()
        .admit_materialized(root)
        .expect("the attack preserves outer bundle syntax and hashes");
    let denial = verify_materialized_backup(
        reopened,
        OfflineInspectionBudget::bounded(4 * 1024, u64::MAX).expect("budget"),
    )
    .expect_err("the artifact owner must reject self-consistent forged bytes");
    let BackupStructuralVerificationDenial::Defects(report) = denial else {
        panic!("owner forgery must be a localized structural defect: {denial:?}");
    };
    assert!(
        report.defects().iter().any(|defect| matches!(
            defect,
            BackupVerificationDefect::OwnerSemanticMismatch { format, kind, .. }
                if *format == artifact_format_for_bundle_family(family)
                    && expected_kind.is_none_or(|expected| *kind == expected)
        )),
        "owner defect was not localized for {family:?}: {:?}",
        report.defects()
    );
    assert!(report.inspected_bytes() <= report.admitted_read_bytes());
    if expected_kind == Some(BackupArtifactSemanticDefectKind::OwnerBindingMismatch) {
        assert!(report.inspected_bytes() < report.admitted_read_bytes());
    }
    assert_eq!(
        report.read_accounting(),
        BackupVerificationReadAccounting::Complete
    );
}

fn rewrite_artifact_and_manifest(
    bundle: &MaterializedBackupBundle,
    target: BackupBundleArtifactFamily,
    forge: impl FnOnce(&mut [u8]),
) {
    let manifest = bundle.manifest();
    let target_row = manifest
        .artifacts()
        .iter()
        .find(|row| row.family() == target)
        .expect("target family");
    let artifact_path = bundle.root().join(target_row.output_name());
    let mut bytes = std::fs::read(&artifact_path).expect("artifact bytes");
    forge(&mut bytes);
    std::fs::write(&artifact_path, &bytes).expect("forged artifact bytes");
    publish_rehashed_manifest(
        bundle.root(),
        manifest,
        target,
        Sha256::digest(&bytes).into(),
    );
}

fn publish_rehashed_manifest(
    root: &std::path::Path,
    manifest: &BackupBundleManifest,
    target: BackupBundleArtifactFamily,
    forged_digest: [u8; 32],
) {
    let rows = manifest
        .artifacts()
        .iter()
        .map(|row| rehashed_row(row, target, forged_digest))
        .collect();
    let forged = BackupBundleManifest::canonical(
        BackupBundleManifestDeclaration::from_manifest_with_artifacts(manifest, rows),
    )
    .expect("attacker can recompute all unauthenticated outer metadata");
    std::fs::write(
        root.join("backup.manifest"),
        BackupBundleFormatAuthority::canonical()
            .encode_manifest(&forged)
            .expect("forged manifest encoding"),
    )
    .expect("forged manifest publication");
}

fn rehashed_row(
    row: &BackupBundleArtifactManifestRow,
    target: BackupBundleArtifactFamily,
    forged_digest: [u8; 32],
) -> BackupBundleArtifactManifestRow {
    if row.family() != target {
        return row.clone();
    }
    BackupBundleArtifactManifestRow::new(
        row.family(),
        row.format(),
        row.identity(),
        row.output_name(),
        row.generation(),
        row.bytes(),
        forged_digest,
        row.coverage().clone(),
        row.reclaim_owner(),
    )
    .expect("outer row remains syntactically coherent")
}

fn forge_owner_bytes(family: BackupBundleArtifactFamily, bytes: &mut [u8]) {
    match family {
        BackupBundleArtifactFamily::RootManifest => {
            bytes[12..20].copy_from_slice(&2_u64.to_le_bytes())
        }
        BackupBundleArtifactFamily::CheckpointManifest => {
            bytes[10..18].copy_from_slice(&2_u64.to_le_bytes());
            refresh_internal_sha256_footer(bytes);
        }
        BackupBundleArtifactFamily::WalSegment => {
            bytes[12..20].copy_from_slice(&999_u64.to_le_bytes());
            refresh_internal_sha256_footer(bytes);
        }
        BackupBundleArtifactFamily::Page | BackupBundleArtifactFamily::Extent => {
            bytes[9..17].copy_from_slice(&2_u64.to_le_bytes());
        }
        BackupBundleArtifactFamily::Index => bytes[1] |= 0b1000_0000,
        BackupBundleArtifactFamily::BlobChunk => {
            let payload_tail = bytes.len() - 33;
            bytes[payload_tail] ^= 0x5a;
            refresh_internal_sha256_footer(bytes);
        }
        BackupBundleArtifactFamily::SecondaryRoot => unreachable!("not in phase 1-6 fixture"),
    }
}

fn refresh_internal_sha256_footer(bytes: &mut [u8]) {
    let footer = bytes.len() - 32;
    let digest = Sha256::digest(&bytes[..footer]);
    bytes[footer..].copy_from_slice(&digest);
}

const fn artifact_format_for_bundle_family(
    family: BackupBundleArtifactFamily,
) -> BackupBundleArtifactFormat {
    match family {
        BackupBundleArtifactFamily::RootManifest => {
            BackupBundleArtifactFormat::PhysicalRootManifestV1
        }
        BackupBundleArtifactFamily::CheckpointManifest => {
            BackupBundleArtifactFormat::RecoveryCheckpointManifestV1
        }
        BackupBundleArtifactFamily::WalSegment => BackupBundleArtifactFormat::WalSegmentV1,
        BackupBundleArtifactFamily::Page => BackupBundleArtifactFormat::PhysicalDataPageV1,
        BackupBundleArtifactFamily::Extent => BackupBundleArtifactFormat::PhysicalExtentRecordV1,
        BackupBundleArtifactFamily::Index => BackupBundleArtifactFormat::LayoutBTreeLeafV1,
        BackupBundleArtifactFamily::BlobChunk => BackupBundleArtifactFormat::BlobChunkV1,
        BackupBundleArtifactFamily::SecondaryRoot => {
            BackupBundleArtifactFormat::PhysicalSecondaryRootManifestV1
        }
    }
}
