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
    registry_of(false)
}

fn registry_of(structural: bool) -> InstalledPersistentNameSchemeRegistry {
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
                structural,
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
    predicted_under(request, false)
}

fn predicted_under(
    request: &PersistentNameBirthRequest,
    structural: bool,
) -> PersistentNameIdentity {
    PersistentNameIdentity::of_pending_birth(
        &request.scheme_id,
        &request.occurrence_key,
        &request.role_path,
        &request.parent_identities,
        &request.canonical_discriminator,
        structural,
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

/// Under a structural-identity scheme the door and minting agree, the
/// identity is the same whichever recorded parents a birth lists, the parents
/// stay recorded in its parts, and the default scheme still tells them apart.
#[test]
fn a_structural_identity_is_independent_of_parents_and_still_records_them() {
    let structural = registry_of(true);
    let roots = [
        request("authored", vec![], 1),
        request("authored", vec![], 2),
    ];
    let minted_roots = PersistentNameMintingAuthority::mint_commit_births_for_test(
        &structural,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &roots,
    )
    .expect("authored births mint");
    let mut live = BTreeMap::new();
    for (root, minted) in roots.iter().zip(&minted_roots) {
        live.insert(minted.identity(), root.holder.clone());
    }
    let (first, second) = (minted_roots[0].identity(), minted_roots[1].identity());
    let mint = |registry: &InstalledPersistentNameSchemeRegistry, parent| {
        PersistentNameMintingAuthority::mint_commit_births_for_test(
            registry,
            &live,
            &BTreeSet::new(),
            &[request("cut_piece", vec![parent], 3)],
        )
        .expect("a birth over a live parent mints")
        .remove(0)
    };
    let (under_first, under_second) = (mint(&structural, first), mint(&structural, second));
    assert_eq!(under_first.identity(), under_second.identity());
    assert_eq!(
        predicted_under(&request("cut_piece", vec![first], 3), true),
        under_first.identity()
    );
    assert_eq!(under_first.parts().parent_identities(), [first]);
    assert!(under_first.parts().structural_identity());
    let default = registry();
    assert_ne!(
        mint(&default, first).identity(),
        mint(&default, second).identity()
    );
}
