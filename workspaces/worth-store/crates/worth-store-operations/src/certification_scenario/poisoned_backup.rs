use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_offline_verifier::{
    verify_materialized_backup, BackupStructuralVerificationDenial, OfflineInspectionBudget,
};
use worth_store_physical_format::{
    BackupBundleArtifactFamily, BackupBundleArtifactManifestRow, BackupBundleFormatAuthority,
    BackupBundleManifest, BackupBundleManifestDeclaration,
};
use worth_store_physical_isolation::BackupReachabilityLeaseRegistry;

use crate::{
    AdmittedOnlineBackup, OperationalControlStorePort, OperationalCounterReceipt,
    OperationalOperationId,
};

pub struct RejectedPoisonedBackupScenario {
    operation: OperationalOperationId,
    omitted_blob_artifact: String,
    torn_wal_artifact: String,
    substituted_index_artifact: String,
    independently_localized_defects: u64,
    rejection_identity: [u8; 32],
    retained_source_leases: u64,
    counters: OperationalCounterReceipt,
}

pub(super) fn reject_poisoned_backup(
    operation: OperationalOperationId,
    admitted: AdmittedOnlineBackup,
    target: &Path,
    control: &impl OperationalControlStorePort,
    leases: &BackupReachabilityLeaseRegistry,
) -> RejectedPoisonedBackupScenario {
    let completion = admitted
        .materialize(target, 64 * 1024, control)
        .expect("owner-opened poisoned backup materialization")
        .finish()
        .expect("owner-completed poisoned backup materialization");
    let counters = OperationalCounterReceipt::from_backup_materialization(&completion)
        .expect("bounded poisoned-backup counters");
    let (materialized, _cut) = completion.into_parts();
    let root = materialized.root().to_path_buf();
    let manifest = materialized.manifest().clone();
    let attack = apply_self_consistent_multi_fault_attack(&root, &manifest);
    drop(materialized);
    let reopened = BackupBundleFormatAuthority::canonical()
        .admit_materialized(&root)
        .expect("attacker-rehashed bundle remains syntactically admissible");
    let denial = verify_materialized_backup(
        reopened,
        OfflineInspectionBudget::bounded(64 * 1024, u64::MAX)
            .expect("bounded poisoned-backup verification"),
    )
    .expect_err("independent verification must reject the multi-fault bundle");
    let independently_localized_defects = localized_defect_count(&denial);
    assert!(
        independently_localized_defects >= 3,
        "the independent verifier must retain all simultaneous fault localizations: {denial:?}"
    );
    let retained_source_leases = leases
        .live_index_snapshot()
        .expect("live backup lease index")
        .active_leases() as u64;
    assert!(
        retained_source_leases > 0,
        "rejection cannot release the cut lease"
    );
    let mut digest = Sha256::new();
    digest.update(b"worth-store-rejected-poisoned-backup-scenario-v2");
    digest.update(operation.stable_fingerprint());
    digest.update(attack.omitted_blob_artifact.as_bytes());
    digest.update(attack.torn_wal_artifact.as_bytes());
    digest.update(attack.substituted_index_artifact.as_bytes());
    digest.update(independently_localized_defects.to_be_bytes());
    digest.update(format!("{denial:?}").as_bytes());
    digest.update(retained_source_leases.to_be_bytes());
    RejectedPoisonedBackupScenario {
        operation,
        omitted_blob_artifact: attack.omitted_blob_artifact,
        torn_wal_artifact: attack.torn_wal_artifact,
        substituted_index_artifact: attack.substituted_index_artifact,
        independently_localized_defects,
        rejection_identity: digest.finalize().into(),
        retained_source_leases,
        counters,
    }
}

struct AppliedBackupAttack {
    omitted_blob_artifact: String,
    torn_wal_artifact: String,
    substituted_index_artifact: String,
}

fn apply_self_consistent_multi_fault_attack(
    root: &Path,
    manifest: &BackupBundleManifest,
) -> AppliedBackupAttack {
    let blob = row_for_family(manifest, BackupBundleArtifactFamily::BlobChunk);
    let wal = row_for_family(manifest, BackupBundleArtifactFamily::WalSegment);
    let index = row_for_family(manifest, BackupBundleArtifactFamily::Index);
    std::fs::remove_file(root.join(blob.output_name())).expect("controlled blob omission");

    let wal_path = root.join(wal.output_name());
    let mut wal_bytes = std::fs::read(&wal_path).expect("materialized WAL bytes");
    wal_bytes.truncate(wal_bytes.len().saturating_sub(7).max(1));
    std::fs::write(&wal_path, &wal_bytes).expect("controlled torn WAL tail");

    let index_path = root.join(index.output_name());
    let index_bytes = worth_store_physical_format::BTreeNodeV1::leaf(
        1,
        vec![worth_store_physical_format::BTreeNodeCellV1::leaf(
            b"catalog-key".to_vec(),
            vec![220],
        )],
        None,
        None,
    )
    .expect("substitute node")
    .encode(4096)
    .expect("substitute node bytes");
    std::fs::write(&index_path, &index_bytes).expect("checksum-valid index substitution");

    let rows = manifest
        .artifacts()
        .iter()
        .map(|row| {
            if row.family() == BackupBundleArtifactFamily::WalSegment {
                rehashed_row(row, &wal_bytes)
            } else if row.family() == BackupBundleArtifactFamily::Index {
                rehashed_row(row, &index_bytes)
            } else {
                row.clone()
            }
        })
        .collect();
    let forged = BackupBundleManifest::canonical(
        BackupBundleManifestDeclaration::from_manifest_with_artifacts(manifest, rows),
    )
    .expect("attacker can recompute unauthenticated outer manifest digests");
    std::fs::write(
        root.join("backup.manifest"),
        BackupBundleFormatAuthority::canonical()
            .encode_manifest(&forged)
            .expect("forged manifest encoding"),
    )
    .expect("forged manifest publication");
    AppliedBackupAttack {
        omitted_blob_artifact: blob.output_name().to_owned(),
        torn_wal_artifact: wal.output_name().to_owned(),
        substituted_index_artifact: index.output_name().to_owned(),
    }
}

fn row_for_family(
    manifest: &BackupBundleManifest,
    family: BackupBundleArtifactFamily,
) -> &BackupBundleArtifactManifestRow {
    manifest
        .artifacts()
        .iter()
        .find(|row| row.family() == family)
        .expect("canonical hostile backup contains every attacked family")
}

fn rehashed_row(
    row: &BackupBundleArtifactManifestRow,
    forged_bytes: &[u8],
) -> BackupBundleArtifactManifestRow {
    BackupBundleArtifactManifestRow::new(
        row.family(),
        row.format(),
        row.identity(),
        row.output_name(),
        row.generation(),
        forged_bytes.len() as u64,
        Sha256::digest(forged_bytes).into(),
        row.coverage().clone(),
        row.reclaim_owner(),
    )
    .expect("outer artifact row remains syntactically coherent")
}

fn localized_defect_count(denial: &BackupStructuralVerificationDenial) -> u64 {
    match denial {
        BackupStructuralVerificationDenial::Defects(report) => report.defects().len() as u64,
        _ => 0,
    }
}

impl RejectedPoisonedBackupScenario {
    pub const fn operation(&self) -> &OperationalOperationId {
        &self.operation
    }

    pub fn omitted_artifact(&self) -> &str {
        &self.omitted_blob_artifact
    }

    pub fn torn_wal_artifact(&self) -> &str {
        &self.torn_wal_artifact
    }

    pub fn substituted_index_artifact(&self) -> &str {
        &self.substituted_index_artifact
    }

    pub const fn independently_localized_defects(&self) -> u64 {
        self.independently_localized_defects
    }

    pub const fn rejection_identity(&self) -> [u8; 32] {
        self.rejection_identity
    }

    pub const fn retained_source_leases(&self) -> u64 {
        self.retained_source_leases
    }

    pub const fn counters(&self) -> OperationalCounterReceipt {
        self.counters
    }
}
