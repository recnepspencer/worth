//! Independent C8 encoding of the selected-route custody transcript.
//! Non-Blob payloads contribute their selected class, tier, and length, but
//! only bounded decoded Blob frames contribute a frame hash.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PhysicalTierClass, SelectedRecordContentClass,
};

const DOMAIN: &[u8] = b"store.physical.released-drop-selected-routes.v1";

pub(super) struct SelectedRouteTranscript(Sha256);

impl SelectedRouteTranscript {
    pub(super) fn new(source_root_generation: u64, row_count: usize) -> Option<Self> {
        let count = u64::try_from(row_count).ok()?;
        let mut digest = Sha256::new();
        digest.update(DOMAIN);
        digest.update(source_root_generation.to_le_bytes());
        digest.update(count.to_le_bytes());
        Some(Self(digest))
    }

    pub(super) fn row(
        &mut self,
        route: CurrentPhysicalRecordPlacement,
        blob_frame_sha256: [u8; 32],
    ) {
        let record = route.record();
        self.0.update(record.allocation_epoch());
        self.0.update(record.ordinal().to_le_bytes());
        self.0
            .update(metadata(route.content_class(), route.tier_class()));
        self.0.update(route.payload_bytes().to_le_bytes());
        self.0.update(blob_frame_sha256);
    }

    pub(super) fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}

fn metadata(class: SelectedRecordContentClass, tier: PhysicalTierClass) -> [u8; 7] {
    let mut bytes = [0; 7];
    match class {
        SelectedRecordContentClass::UnknownLegacy => {}
        SelectedRecordContentClass::Opaque => bytes[0] = 1,
        SelectedRecordContentClass::Blob(kind) => {
            bytes[0] = 2;
            bytes[1] = kind as u8;
        }
        SelectedRecordContentClass::BTreeNode { family_code } => {
            bytes[0] = 3;
            bytes[2..4].copy_from_slice(&family_code.to_le_bytes());
        }
        SelectedRecordContentClass::DerivedDirectory => bytes[0] = 4,
    }
    bytes[4] = match tier {
        PhysicalTierClass::Primary => 0,
        PhysicalTierClass::Hot => 1,
        PhysicalTierClass::Cold => 2,
    };
    bytes
}
