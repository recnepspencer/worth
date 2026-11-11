use super::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactFormat,
    BackupBundleArtifactManifestRow, BackupBundleManifest, BackupBundleManifestDeclaration,
    BackupBundleManifestIdentity, BackupBundleRecoveryCoordinates,
};
use crate::{
    BackupBundleFormatAuthority, BackupBundlePhysicalOwner, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationOwner,
};
use worth_foundational::CanonicalDigestId;

#[test]
fn top_level_record_extent_owner_round_trips_without_a_fabricated_segment() {
    let generation = PhysicalGeneration::from_raw(1).expect("generation");
    let owner = PhysicalGenerationOwner::for_record_extent(
        PhysicalExtentId::from_raw(1).expect("extent"),
        generation,
    );
    let row = BackupBundleArtifactManifestRow::new(
        BackupBundleArtifactFamily::Extent,
        BackupBundleArtifactFormat::PhysicalExtentRecordV1,
        "record-extent-a",
        "record-extent-a.bin",
        1,
        4,
        [7; 32],
        BackupBundleArtifactCoverage::PhysicalReachability,
        BackupBundlePhysicalOwner::from_generation_owner(owner),
    )
    .expect("top-level record extent row");
    let manifest = BackupBundleManifest::canonical(BackupBundleManifestDeclaration::new(
        BackupBundleManifestIdentity {
            cut_identity: CanonicalDigestId::new([1; 32]),
            store_lineage: "lineage".to_owned(),
            root_generation: 1,
            manifest_generation: 1,
        },
        BackupBundleRecoveryCoordinates {
            checkpoint_identity: "checkpoint".to_owned(),
            durable_checkpoint_lsn: 1,
            wal_half_open_interval: (1, 1),
            acknowledged_frontier: 1,
        },
        9,
        vec![row],
    ))
    .expect("canonical record extent manifest");
    let authority = BackupBundleFormatAuthority::canonical();
    let encoded = authority
        .encode_manifest(&manifest)
        .expect("encode manifest");
    let decoded = authority
        .decode_manifest(&encoded)
        .expect("decode manifest");

    assert_eq!(decoded, manifest);
}
