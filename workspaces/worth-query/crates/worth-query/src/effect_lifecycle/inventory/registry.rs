use crate::basis_lifecycle::BasisFamily;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::super::inventory_rows::{
    effect_lifecycle_family_rows, effect_lifecycle_public_surface_rows,
};
use super::super::support_matrix::EffectLifecycleSupportRow;
use super::super::taxonomy::EffectFamily;
use super::family::{EffectLifecycleFamilyInventory, EffectLifecycleFamilyInventoryRow};
use super::kinds::EffectLifecycleFamilyKey;
use super::public_surface::EffectLifecyclePublicSurfaceInventory;
use super::EFFECT_LIFECYCLE_IDENTITY_SCOPE;

fn compose_inventory_identity(
    identity_family: &str,
    rows: &[WorthQueryEvidenceIdentity],
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            identity_family,
        )
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("rows"), rows)
        .seal().expect("closed effect inventories contain four family rows or nine surface rows with fixed labels and sealed row identities")
}

pub fn effect_lifecycle_family_inventory() -> EffectLifecycleFamilyInventory {
    let rows = effect_lifecycle_family_rows();
    let row_identities = rows
        .iter()
        .map(|row| row.row_identity().clone())
        .collect::<Vec<_>>();
    let inventory_identity =
        compose_inventory_identity("effect_lifecycle_family_inventory_v1", &row_identities);
    EffectLifecycleFamilyInventory {
        rows,
        inventory_identity,
    }
}

pub fn effect_lifecycle_public_surface_inventory() -> EffectLifecyclePublicSurfaceInventory {
    let rows = effect_lifecycle_public_surface_rows();
    let row_identities = rows
        .iter()
        .map(|row| row.row_identity().clone())
        .collect::<Vec<_>>();
    let inventory_identity = compose_inventory_identity(
        "effect_lifecycle_public_surface_inventory_v1",
        &row_identities,
    );
    EffectLifecyclePublicSurfaceInventory {
        rows,
        inventory_identity,
    }
}

pub fn effect_lifecycle_family_row_for_key(
    family_key: EffectLifecycleFamilyKey,
) -> Option<EffectLifecycleFamilyInventoryRow> {
    effect_lifecycle_family_inventory()
        .rows
        .into_iter()
        .find(|row| row.family_key() == family_key)
}

pub fn effect_lifecycle_family_row_for(
    effect_family: EffectFamily,
) -> Option<EffectLifecycleFamilyInventoryRow> {
    let family_key = match effect_family {
        EffectFamily::Mutation => EffectLifecycleFamilyKey::Mutation,
        EffectFamily::Merge => EffectLifecycleFamilyKey::Merge,
        EffectFamily::Writeback => EffectLifecycleFamilyKey::Writeback,
    };
    effect_lifecycle_family_row_for_key(family_key)
}

pub fn effect_lifecycle_supported_basis_families(effect_family: EffectFamily) -> Vec<BasisFamily> {
    effect_lifecycle_family_row_for(effect_family)
        .map(|row| row.admitted_basis_families().to_vec())
        .unwrap_or_default()
}

pub fn effect_lifecycle_support_row_matches_inventory(row: &EffectLifecycleSupportRow) -> bool {
    let Some(family_row) = effect_lifecycle_family_row_for(row.effect_family()) else {
        return false;
    };
    row.authority_owner() == family_row.authority_owner()
        && row.lowered_artifact_kind() == family_row.lowered_artifact_kind()
        && row.receipt_artifact_kind() == family_row.receipt_artifact_kind()
}
