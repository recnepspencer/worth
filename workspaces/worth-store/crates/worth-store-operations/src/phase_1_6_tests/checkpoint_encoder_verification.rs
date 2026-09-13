use sha2::{Digest, Sha256};

use super::support::*;

#[test]
fn independent_wire_oracle_validates_the_emitted_checkpoint_encoder_bytes() {
    let scenario = BackupScenario::new("independent-checkpoint-encoder");
    let authority = scenario.authority();
    let control = scenario.control_store();
    let completion = OnlineBackupIntent::new(
        OperationalOperationId::new("backup-independent-checkpoint-encoder").expect("operation id"),
        scenario.coordinates(),
        scenario.cut_manifest(),
        backup_custody(&authority),
    )
    .admit_cut(&authority, &control, &scenario.leases)
    .expect("cut")
    .materialize(&scenario.target, 19, &control)
    .expect("materialization session")
    .finish()
    .expect("materialization");
    let (materialized, _) = completion.into_parts();
    let row = materialized
        .manifest()
        .artifacts()
        .iter()
        .find(|row| row.format() == BackupBundleArtifactFormat::RecoveryCheckpointManifestV1)
        .expect("checkpoint manifest row");
    let bytes = std::fs::read(materialized.root().join(row.output_name()))
        .expect("emitted checkpoint bytes");

    assert!(bytes.len() >= 78 + 32);
    assert_eq!(&bytes[0..8], b"WORTHCKP");
    assert_eq!(read_u16(&bytes, 8), 1);
    assert_eq!(read_u64(&bytes, 10), 1);
    assert_eq!(read_u64(&bytes, 18), 10);
    assert_eq!(read_u64(&bytes, 26), 1);
    assert_eq!(read_u64(&bytes, 34), 1);
    assert_eq!(read_u64(&bytes, 42), 10);
    assert_eq!(read_u64(&bytes, 50), 12);
    assert_eq!(read_u64(&bytes, 58), 10);
    assert_eq!(read_u64(&bytes, 66), 1);
    assert_eq!(read_u64(&bytes, 78), 200);
    assert_eq!(read_u64(&bytes, 86), 4);
    assert_eq!(read_u64(&bytes, 94), 1);
    assert_eq!(read_u64(&bytes, 102), 10);
    let identity_bytes = read_u32(&bytes, 74) as usize;
    let identity_start = 78 + 32;
    let identity_end = identity_start + identity_bytes;
    assert_eq!(
        &bytes[identity_start..identity_end],
        scenario.checkpoint_identity().as_bytes()
    );
    let footer_start = bytes.len() - 32;
    assert_eq!(
        Sha256::digest(&bytes[..footer_start]).as_slice(),
        &bytes[footer_start..]
    );
    let expected_digest = row.content_digest();
    assert_eq!(
        Sha256::digest(&bytes).as_slice(),
        expected_digest.as_slice()
    );
    assert_eq!(bytes.len(), identity_end + 32);
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("fixed field"))
}
