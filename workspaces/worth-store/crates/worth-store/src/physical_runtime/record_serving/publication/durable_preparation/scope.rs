use sha2::{Digest, Sha256};

use super::PhysicalManifestCapacityTransition;
use crate::physical_runtime::{AdmittedPhysicalRecordFormat, AdmittedRecordPlacementPolicy};

const SCOPE_DOMAIN: &[u8] = b"store.physical.record-append.scope.v2";

pub(in crate::physical_runtime::record_serving) fn record_append_scope_identity(
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    manifest_capacity_transition: PhysicalManifestCapacityTransition,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    write_field(&mut digest, SCOPE_DOMAIN);
    write_field(
        &mut digest,
        &format.declaration().canonical_identity_bytes(),
    );
    write_field(&mut digest, &placement.segment_pages().get().to_le_bytes());
    write_field(
        &mut digest,
        &placement.extent_threshold().get().to_le_bytes(),
    );
    write_field(&mut digest, &placement.page_fill().get().to_le_bytes());
    write_field(
        &mut digest,
        &placement.manifest_capacity().get().to_le_bytes(),
    );
    write_field(&mut digest, &[manifest_capacity_transition.identity_code()]);
    write_field(&mut digest, &placement.arena_capacity().get().to_le_bytes());
    write_field(
        &mut digest,
        &placement.arena_evacuation().get().to_le_bytes(),
    );
    write_field(
        &mut digest,
        &placement.arena_index_bytes().get().to_le_bytes(),
    );
    digest.finalize().into()
}

fn write_field(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}
