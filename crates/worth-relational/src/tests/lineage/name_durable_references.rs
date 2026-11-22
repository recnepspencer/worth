//! Arch 10, 21: a persisted birth whose canonical discriminator embeds other
//! names (`TAG_NAME_REF`) reopens only when every embedded name was itself
//! born, earlier in the log or anywhere in the same commit, including a
//! birth that appears later in that commit's event order.

use std::collections::{BTreeMap, BTreeSet};

use crate::facade::lineage::{
    admit_persisted_name, encode_durable_persistent_name_log, rebuild_name_index_from_durable_log,
    LineageEventKind, LineagePostureClaim, LineageRecordTarget, PersistentNameBirthRequest,
    PersistentNameBranchIndex, PersistentNameDiscriminatorReference, PersistentNameDurabilityError,
    PersistentNameHolder, PersistentNameIdentity, PersistentNameMintingAuthority,
    PersistentNameReferenceBasis, PersistentNameReferenceRole, PersistentNameSubjectKind,
};
use crate::identity::data::EntityId;

use super::name_durable_world::durable_naming_history;
use super::name_index_fixtures::main_branch;
use super::name_world::{LineageWorld, Name, Subject};
use super::posture_fixtures::{cad_scheme_registry, face_birth};

/// A canonical discriminator made only of embedded name references.
fn referencing_discriminator(references: &[Name]) -> Vec<u8> {
    let mut bytes = b"worth.query.name-discriminator.v1\0".to_vec();
    let discriminator_id = b"worth.test.durable.references.v1";
    bytes.extend_from_slice(&(discriminator_id.len() as u64).to_be_bytes());
    bytes.extend_from_slice(discriminator_id);
    for reference in references {
        bytes.push(0x20);
        bytes.extend_from_slice(reference.as_bytes());
    }
    bytes
}

fn entity_of(holder: &PersistentNameHolder) -> EntityId {
    match holder {
        PersistentNameHolder::Entity(entity) => *entity,
        _ => unreachable!("the reference world creates entities"),
    }
}

/// The identity minting will give `request`, which never depends on holders.
fn minted_identity(request: &PersistentNameBirthRequest) -> Name {
    PersistentNameMintingAuthority::mint_commit_births(
        &cad_scheme_registry(),
        &BTreeMap::new(),
        &BTreeSet::new(),
        std::slice::from_ref(request),
    )
    .expect("an authored birth mints")[0]
        .identity()
}

#[test]
fn births_embedding_earlier_and_later_same_commit_names_reopen_identically() {
    let mut world = LineageWorld::new();
    let basis = world.create(Subject::Entity, "basis");
    world.commit();
    let basis = world.name_of(&basis);

    let referencing = world.new_record(Subject::Entity);
    let anchor = world.new_record(Subject::Entity);
    assert!(entity_of(&referencing) < entity_of(&anchor));
    let anchor_birth = face_birth("occ.refs", "authored", vec![], "anchor", entity_of(&anchor));
    let anchor_name = minted_identity(&anchor_birth);
    let mut referencing_birth =
        face_birth("occ.refs", "cut_piece", vec![], "", entity_of(&referencing));
    referencing_birth.canonical_discriminator = referencing_discriminator(&[basis, anchor_name]);
    referencing_birth.referenced_names = vec![
        PersistentNameDiscriminatorReference {
            identity: basis,
            basis: PersistentNameReferenceBasis::Basis,
        },
        PersistentNameDiscriminatorReference {
            identity: anchor_name,
            basis: PersistentNameReferenceBasis::SameOccurrence {
                role: "authored".to_owned(),
            },
        },
    ];
    for (holder, birth) in [(&referencing, referencing_birth), (&anchor, anchor_birth)] {
        world.postures.push(LineagePostureClaim::Create {
            target: LineageRecordTarget::Entity(entity_of(holder)),
            birth,
        });
    }
    let batch = world.commit();
    let births: Vec<Name> = batch
        .lineage_events
        .iter()
        .filter(|event| event.kind() == LineageEventKind::Create)
        .filter_map(|event| event.name_birth().map(|birth| birth.identity()))
        .collect();
    let referencing_name = world.name_of(&referencing);
    assert_eq!(
        births,
        vec![referencing_name, anchor_name],
        "the referencing birth precedes the same-commit birth it embeds"
    );

    let bytes = encode_durable_persistent_name_log(
        &main_branch(),
        world.log().iter().map(|batch| (batch, None)),
    )
    .unwrap();
    let reopened =
        rebuild_name_index_from_durable_log(&bytes).expect("every embedded name was born");
    let live =
        PersistentNameBranchIndex::rebuild_from_log(main_branch(), None, world.log()).unwrap();
    assert_eq!(reopened, live);
    for name in [basis, anchor_name, referencing_name] {
        let key = admit_persisted_name(
            &reopened,
            PersistentNameSubjectKind::Entity,
            *name.as_bytes(),
        )
        .expect("a verified persisted birth readmits");
        assert_eq!(key.digest(), name.as_bytes());
    }
}

#[test]
fn a_digest_consistent_birth_embedding_an_unborn_name_refuses_typed() {
    let history = durable_naming_history();
    let unborn = PersistentNameIdentity::from_minted_digest([0xcd; 32]);
    let holder = history.commits[1]
        .0
        .entity_holders
        .values()
        .next()
        .copied()
        .unwrap();
    let mut request = face_birth("occ.forged", "authored", vec![], "", holder);
    request.canonical_discriminator = referencing_discriminator(&[unborn]);
    request.referenced_names = vec![PersistentNameDiscriminatorReference {
        identity: unborn,
        basis: PersistentNameReferenceBasis::Basis,
    }];
    let minted = PersistentNameMintingAuthority::mint_commit_births(
        &cad_scheme_registry(),
        &BTreeMap::from([(unborn, PersistentNameHolder::Entity(holder))]),
        &BTreeSet::new(),
        &[request],
    )
    .expect("the minting authority mints over a name it was told is live")
    .remove(0);
    let refused = history.rebuild_mutated(|commits| {
        let event = commits[1]
            .0
            .lineage_events
            .iter_mut()
            .find(|event| event.name_birth().is_some())
            .unwrap();
        event.name_birth = Some(minted);
    });
    assert_eq!(
        refused,
        Err(PersistentNameDurabilityError::UnverifiedBareIdentity {
            raw: *unborn.as_bytes(),
            role: PersistentNameReferenceRole::DiscriminatorReference,
        })
    );
}
