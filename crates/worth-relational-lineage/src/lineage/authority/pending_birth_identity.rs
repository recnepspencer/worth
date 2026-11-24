//! Agreement test between `PersistentNameIdentity::of_pending_birth` and
//! `PersistentNameMintingAuthority::mint_commit_births`.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::lineage::data::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameBirthRequest, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameSubjectKind,
};
use crate::lineage::authority::PersistentNameMintingAuthority;

fn registry() -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                "worth.cad.face.v1",
                PersistentNameSubjectKind::Entity,
                "face",
                ["authored", "cut_piece"],
                false,
                true,
            )
            .expect("valid face scheme"),
        )
        .expect("register face scheme");
    registry
}

fn request(
    role: &str,
    parents: Vec<PersistentNameIdentity>,
    slot: u64,
) -> PersistentNameBirthRequest {
    PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: format!("occ.{slot}"),
        role_path: role.to_owned(),
        parent_identities: parents,
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
        holder: PersistentNameHolder::Entity(
            crate::contracts::identity::data::record_ids::EntityId::new(
                crate::contracts::identity::data::primitives::PartitionId::main(),
                slot,
                1,
            ),
        ),
    }
}

fn predicted(request: &PersistentNameBirthRequest) -> PersistentNameIdentity {
    PersistentNameIdentity::of_pending_birth(
        &request.scheme_id,
        &request.occurrence_key,
        &request.role_path,
        &request.parent_identities,
        &request.canonical_discriminator,
    )
}

/// The door and commit minting agree on a parentless birth and on a
/// birth whose parents arrive in either order.
#[test]
fn a_pending_birth_identity_is_the_identity_minting_gives() {
    let registry = registry();
    let roots = [
        request("authored", vec![], 1),
        request("authored", vec![], 2),
    ];
    let minted_roots = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &roots,
    )
    .expect("authored births mint");
    let mut live = BTreeMap::new();
    for (root, minted) in roots.iter().zip(&minted_roots) {
        assert_eq!(predicted(root), minted.identity());
        live.insert(minted.identity(), root.holder.clone());
    }
    let parents = [minted_roots[0].identity(), minted_roots[1].identity()];
    let forward = request("cut_piece", parents.to_vec(), 3);
    let reversed = request("cut_piece", vec![parents[1], parents[0]], 3);
    let minted = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &registry,
        &live,
        &BTreeSet::new(),
        std::slice::from_ref(&forward),
    )
    .expect("a birth over live parents mints");
    assert_eq!(predicted(&forward), minted[0].identity());
    assert_eq!(predicted(&reversed), minted[0].identity());
    assert_ne!(predicted(&forward), predicted(&roots[0]));
}
