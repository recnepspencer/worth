use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PersistedRecordIdentity, PhysicalTierClass, SelectedRecordContentClass,
};

use super::inventory::SelectedReleaseFact;

pub(super) const ROUTES_DOMAIN: &[u8] = b"store.physical.released-drop-selected-routes.v1";
pub(super) const CLOSURE_DOMAIN: &[u8] = b"store.physical.released-drop-closure.v1";
pub(super) const EXTERNAL_DOMAIN: &[u8] = b"store.physical.released-drop-external-edges.v1";
pub(super) const POSTORDER_DOMAIN: &[u8] = b"store.physical.released-drop-postorder.v1";

pub(super) fn record_id(hasher: &mut Sha256, record: PersistedRecordIdentity) {
    hasher.update(record.allocation_epoch());
    hasher.update(record.ordinal().to_le_bytes());
}

pub(super) fn selected_routes(root_generation: u64, facts: &[SelectedReleaseFact]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(ROUTES_DOMAIN);
    hasher.update(root_generation.to_le_bytes());
    hasher.update((facts.len() as u64).to_le_bytes());
    for fact in facts {
        record_id(&mut hasher, fact.record);
        hasher.update(route_metadata(fact.class, fact.tier));
        hasher.update(fact.payload_bytes.to_le_bytes());
        // Only Blob payloads carry liveness edges. The full selected Blob
        // frame is bounded and authenticated; non-Blob frame bytes are not
        // read merely to certify an unrelated generation's release.
        hasher.update(fact.frame_sha256);
    }
    hasher.finalize().into()
}

fn route_metadata(class: SelectedRecordContentClass, tier: PhysicalTierClass) -> [u8; 7] {
    let mut encoded = [0; 7];
    match class {
        SelectedRecordContentClass::UnknownLegacy => {}
        SelectedRecordContentClass::Opaque => encoded[0] = 1,
        SelectedRecordContentClass::Blob(kind) => {
            encoded[0] = 2;
            encoded[1] = kind as u8;
        }
        SelectedRecordContentClass::BTreeNode { family_code } => {
            encoded[0] = 3;
            encoded[2..4].copy_from_slice(&family_code.to_le_bytes());
        }
        SelectedRecordContentClass::DerivedDirectory => encoded[0] = 4,
    }
    encoded[4] = match tier {
        PhysicalTierClass::Primary => 0,
        PhysicalTierClass::Hot => 1,
        PhysicalTierClass::Cold => 2,
    };
    encoded
}
