use sha2::{Digest, Sha256};

use super::super::registry::PhysicalMutationBindingBasis;
use super::PhysicalMutationBindingCompactionDenial;
use crate::physical_runtime::record_serving::SelectedBlobManifestPins;

const DROP_MATERIAL_DOMAIN: &[u8] = b"worth.store.blob.reclaim.mutation.v1";

pub(in crate::physical_runtime::durability::mutation::idempotency) fn drop_material(
    store: [u8; 16],
    attempt: [u8; 16],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(DROP_MATERIAL_DOMAIN);
    digest.update(store);
    digest.update(attempt);
    digest.update([2]);
    digest.finalize().into()
}

pub(super) fn drop_materials(
    selected: &SelectedBlobManifestPins,
) -> Result<Vec<[u8; 32]>, PhysicalMutationBindingCompactionDenial> {
    let mut materials = Vec::new();
    materials
        .try_reserve_exact(selected.pins().len())
        .map_err(|_| PhysicalMutationBindingCompactionDenial::BindingBytesExceeded)?;
    for pin in selected.pins() {
        materials.push(drop_material(pin.store(), pin.attempt()));
    }
    materials.sort_unstable();
    if materials.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(PhysicalMutationBindingCompactionDenial::RegistryChanged);
    }
    Ok(materials)
}

pub(super) fn pins_terminal_fate(
    basis: &PhysicalMutationBindingBasis,
    materials: &[[u8; 32]],
) -> bool {
    materials
        .binary_search(&basis.key().caller_material().bytes())
        .is_ok()
}
