mod basis_tests;
mod discriminator_scan_tests;
mod invariance_proptest;

use std::collections::{BTreeMap, BTreeSet};
use std::mem::size_of;

use crate::identity::data::{EntityId, PartitionId};
use crate::lineage::authority::PersistentNameMintingAuthority;
use crate::lineage::data::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, LineageNameDenial,
    PersistentNameBirthRequest, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameSubjectKind,
};

pub(super) fn face_registry() -> InstalledPersistentNameSchemeRegistry {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(InstalledPersistentNameScheme::from_ordered_roles(
            "worth.cad.face.v1",
            PersistentNameSubjectKind::Entity,
            ["authored", "cut_piece", "seam_patch"],
            true,
            true,
        ))
        .expect("register face scheme");
    registry
}

pub(super) fn entity_holder(slot: u32, generation: u32) -> PersistentNameHolder {
    PersistentNameHolder::Entity(EntityId::new(
        PartitionId::main(),
        u64::from(slot),
        generation,
    ))
}

/// Encodes a canonical `worth.query.name-discriminator.v1\0` payload carrying
/// one `TAG_TEXT` label so Relational tests exercise the strict canonical wire.
pub(super) fn canonical_label_discriminator(label: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"worth.query.name-discriminator.v1\0");
    let disc_id = b"worth.test.label.v1";
    bytes.extend_from_slice(&(disc_id.len() as u64).to_be_bytes());
    bytes.extend_from_slice(disc_id);
    bytes.push(0x19); // TAG_TEXT
    bytes.extend_from_slice(&(label.len() as u64).to_be_bytes());
    bytes.extend_from_slice(label.as_bytes());
    bytes
}

#[test]
fn bounded_size_holds_across_split_depths_1_16_and_64() {
    assert_eq!(
        size_of::<PersistentNameIdentity>(),
        PersistentNameIdentity::BYTE_LEN
    );
    assert_eq!(PersistentNameIdentity::BYTE_LEN, 32);

    let registry = face_registry();
    let mut live_holders = BTreeMap::new();
    let empty_retired = BTreeSet::new();

    let root_records = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live_holders,
        &empty_retired,
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.root".to_owned(),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            holder: entity_holder(0, 1),
        }],
    )
    .expect("root birth succeeds");
    let mut current = root_records[0].identity();
    live_holders.insert(current, entity_holder(0, 1));

    let mut depth_1_part_bytes = None;
    let mut depth_16_part_bytes = None;
    let mut depth_64_part_bytes = None;

    for depth in 1..=64u32 {
        let mut retired = BTreeSet::new();
        retired.insert(current);
        let split_disc = canonical_label_discriminator(&format!("side:{depth:02}"));
        let req = PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: format!("occ.cut.{depth:02}"),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![current],
            referenced_names: Vec::new(),
            canonical_discriminator: split_disc,
            holder: entity_holder(depth, 1),
        };
        let minted = PersistentNameMintingAuthority::mint_commit_births(
            &registry,
            &live_holders,
            &retired,
            &[req],
        )
        .expect("split depth mint succeeds");
        assert_eq!(minted.len(), 1);
        let record = &minted[0];
        assert_eq!(record.identity().as_bytes().len(), 32);
        assert_eq!(record.parts().parent_identities().len(), 1);
        let bytes = record.parts().logical_byte_len();
        match depth {
            1 => depth_1_part_bytes = Some(bytes),
            16 => depth_16_part_bytes = Some(bytes),
            64 => depth_64_part_bytes = Some(bytes),
            _ => {}
        }
        live_holders.remove(&current);
        current = record.identity();
        live_holders.insert(current, entity_holder(depth, 1));
    }

    assert_eq!(
        depth_1_part_bytes, depth_16_part_bytes,
        "birth part size must remain O(1) between split depth 1 and 16"
    );
    assert_eq!(
        depth_16_part_bytes, depth_64_part_bytes,
        "birth part size must remain O(1) between split depth 16 and 64"
    );
}

#[test]
fn same_commit_and_live_holder_collisions_refuse_duplicate_birth() {
    let registry = face_registry();
    let mut live_holders = BTreeMap::new();
    let empty_retired = BTreeSet::new();

    let first_holder = entity_holder(10, 1);
    let second_holder = entity_holder(11, 1);
    let sibling_a = PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: "occ.split.1".to_owned(),
        role_path: "cut_piece".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: canonical_label_discriminator("same-side"),
        holder: first_holder.clone(),
    };
    let sibling_b = PersistentNameBirthRequest {
        holder: second_holder.clone(),
        ..sibling_a.clone()
    };
    let denial = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live_holders,
        &empty_retired,
        &[sibling_a.clone(), sibling_b],
    )
    .expect_err("colliding same-commit siblings must refuse");
    match denial {
        LineageNameDenial::DuplicateBirth { first, second, .. } => {
            assert_eq!(first, first_holder);
            assert_eq!(second, second_holder);
        }
        other => panic!("expected DuplicateBirth, got {other:?}"),
    }

    let committed = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live_holders,
        &empty_retired,
        std::slice::from_ref(&sibling_a),
    )
    .expect("first birth commits");
    let committed_id = committed[0].identity();
    live_holders.insert(committed_id, first_holder.clone());

    let challenger_holder = entity_holder(25, 2);
    let challenger = PersistentNameBirthRequest {
        holder: challenger_holder.clone(),
        ..sibling_a
    };
    let live_denial = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live_holders,
        &empty_retired,
        &[challenger],
    )
    .expect_err("colliding with unretired live holder must refuse");
    assert_eq!(
        live_denial,
        LineageNameDenial::DuplicateBirth {
            identity: committed_id,
            first: first_holder,
            second: challenger_holder,
        }
    );
}

#[test]
fn independent_reorder_keeps_identical_names_while_parent_change_mints_distinct_name() {
    let registry = face_registry();
    let f1 = PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: "occ.f1".to_owned(),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
        holder: entity_holder(1, 1),
    };
    let f2 = PersistentNameBirthRequest {
        scheme_id: "worth.cad.face.v1".to_owned(),
        occurrence_key: "occ.f2".to_owned(),
        role_path: "authored".to_owned(),
        parent_identities: Vec::new(),
        referenced_names: Vec::new(),
        canonical_discriminator: Vec::new(),
        holder: entity_holder(2, 1),
    };

    let order_12 = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[f1.clone(), f2.clone()],
    )
    .unwrap();
    let order_21 = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &[
            PersistentNameBirthRequest {
                holder: entity_holder(99, 7),
                ..f2
            },
            PersistentNameBirthRequest {
                holder: entity_holder(42, 3),
                ..f1
            },
        ],
    )
    .unwrap();

    assert_eq!(order_12[0].identity(), order_21[1].identity());
    assert_eq!(order_12[1].identity(), order_21[0].identity());

    let mut live = BTreeMap::new();
    live.insert(order_12[0].identity(), entity_holder(1, 1));
    live.insert(order_12[1].identity(), entity_holder(2, 1));

    let cut_f1 = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.cut".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![order_12[0].identity()],
            referenced_names: Vec::new(),
            canonical_discriminator: canonical_label_discriminator("left"),
            holder: entity_holder(3, 1),
        }],
    )
    .unwrap();
    let cut_f2 = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.cut".to_owned(),
            role_path: "cut_piece".to_owned(),
            parent_identities: vec![order_12[1].identity()],
            referenced_names: Vec::new(),
            canonical_discriminator: canonical_label_discriminator("left"),
            holder: entity_holder(3, 1),
        }],
    )
    .unwrap();
    assert_ne!(
        cut_f1[0].identity(),
        cut_f2[0].identity(),
        "changing a split's parent must mint a distinct identity"
    );

    let merge_ab = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.merge".to_owned(),
            role_path: "seam_patch".to_owned(),
            parent_identities: vec![order_12[0].identity(), order_12[1].identity()],
            referenced_names: Vec::new(),
            canonical_discriminator: canonical_label_discriminator("union"),
            holder: entity_holder(4, 1),
        }],
    )
    .unwrap();
    let merge_ba = PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &live,
        &BTreeSet::new(),
        &[PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: "occ.merge".to_owned(),
            role_path: "seam_patch".to_owned(),
            parent_identities: vec![order_12[1].identity(), order_12[0].identity()],
            referenced_names: Vec::new(),
            canonical_discriminator: canonical_label_discriminator("union"),
            holder: entity_holder(88, 9),
        }],
    )
    .unwrap();
    assert_eq!(merge_ab[0].identity(), merge_ba[0].identity());
}
