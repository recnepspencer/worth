use super::{u32_at, u64_at, ArtifactGranule, ArtifactInventory, FrameGrammar};
use std::num::NonZeroU64;
use worth_store_physical_format::{
    decode_checkpoint_certificate, PhysicalArtifactReadTarget, PhysicalCheckpointIdentity,
    WalSegmentIdentity,
};
use worth_store_physical_integrity::{
    CheckpointStreamHeaderScopeIdentity, PhysicalArtifactScope, PhysicalByteRange,
};

pub(super) fn collect(inventory: &mut ArtifactInventory) {
    let paths = inventory
        .manifest
        .paths()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    for path in paths {
        let bytes = std::fs::read(inventory.root.join(&path)).unwrap();
        if bytes.starts_with(b"WORTHWAL") {
            let mut offset = 0;
            while offset < bytes.len() {
                let length = 148 + u64_at(&bytes, offset + 44) as usize;
                let identity = WalSegmentIdentity::new(
                    u64_at(&bytes, offset + 12),
                    u64_at(&bytes, offset + 20),
                )
                .unwrap();
                let scope = PhysicalArtifactScope::wal_frame(
                    inventory.store,
                    identity,
                    PhysicalByteRange::new(offset as u64, length as u64).unwrap(),
                );
                inventory.push(ArtifactGranule {
                    path: path.clone(),
                    family: "wal_frame",
                    scope,
                    target: PhysicalArtifactReadTarget::Wal(identity),
                    grammar: FrameGrammar::Wal,
                });
                offset += length;
            }
            assert_eq!(offset, bytes.len());
        } else if bytes.starts_with(b"WCP7REC\0") {
            let identity = PhysicalCheckpointIdentity::new(
                inventory.store,
                NonZeroU64::new(u64_at(&bytes, 32)).unwrap(),
            );
            let mut offset = 0;
            let mut certificates_start = None;
            while offset < bytes.len() {
                let length = 20 + u32_at(&bytes, offset + 12) as usize;
                if matches!(bytes[offset + 9], 6 | 7) {
                    // Custody certificates are validated through the footer's
                    // aggregate basis; they have no independently scrubbed scope.
                    // Decode only this immutable lawful baseline, never damaged bytes.
                    decode_checkpoint_certificate(&bytes[offset..offset + length])
                        .expect("canonical production checkpoint certificate");
                    certificates_start.get_or_insert(offset);
                    offset += length;
                    continue;
                }
                if bytes[offset + 9] == 5
                    && let Some(start) = certificates_start
                {
                    inventory.checkpoint_certificates.insert(
                        identity.sequence().get(),
                        PhysicalByteRange::new(start as u64, (offset - start) as u64).unwrap(),
                    );
                }
                let range = PhysicalByteRange::new(offset as u64, length as u64).unwrap();
                let (family, scope) = match bytes[offset + 9] {
                    1 => (
                        "checkpoint_stream_header",
                        PhysicalArtifactScope::checkpoint_stream_header(
                            CheckpointStreamHeaderScopeIdentity::known(identity),
                            range,
                        ),
                    ),
                    2 => (
                        "checkpoint_dirty_basis",
                        PhysicalArtifactScope::checkpoint_dirty_basis(identity, range),
                    ),
                    3 => (
                        "checkpoint_binding_compaction",
                        PhysicalArtifactScope::checkpoint_binding_compaction(identity, range),
                    ),
                    4 => (
                        "checkpoint_binding",
                        PhysicalArtifactScope::checkpoint_binding(identity, range),
                    ),
                    5 => (
                        "checkpoint_footer",
                        PhysicalArtifactScope::checkpoint_footer(identity, range),
                    ),
                    _ => panic!("canonical production checkpoint kind"),
                };
                inventory.push(ArtifactGranule {
                    path: path.clone(),
                    family,
                    scope,
                    target: PhysicalArtifactReadTarget::Checkpoint(identity),
                    grammar: FrameGrammar::Checkpoint,
                });
                offset += length;
            }
            assert_eq!(offset, bytes.len());
        }
    }
}
