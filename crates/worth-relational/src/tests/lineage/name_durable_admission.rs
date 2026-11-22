//! Arch 10, 21: a fresh installation reopened from the durable log admits a
//! retained 32-byte name only when the log holds that name's verified birth,
//! and every persisted reference must name such a birth.

use std::collections::BTreeMap;

use crate::facade::lineage::{
    admit_persisted_name, rebuild_name_index_from_durable_log, LineageEventKind,
    PersistentNameDurabilityError, PersistentNameHolder, PersistentNameIdentity, PersistentNameKey,
    PersistentNameMintingAuthority, PersistentNameReferenceRole, PersistentNameSubjectKind,
};
use crate::identity::data::{EntityId, PartitionId};

use super::name_durable_world::durable_naming_history;
use super::posture_fixtures::{cad_scheme_registry, face_birth};

fn fabricated() -> PersistentNameIdentity {
    PersistentNameIdentity::from_minted_digest([0xab; 32])
}

fn unverified(
    raw: PersistentNameIdentity,
    role: PersistentNameReferenceRole,
) -> PersistentNameDurabilityError {
    PersistentNameDurabilityError::UnverifiedBareIdentity {
        raw: *raw.as_bytes(),
        role,
    }
}

#[test]
fn fresh_installation_readmits_every_recorded_name_and_refuses_bare_identities() {
    let history = durable_naming_history();
    let live = history.live_index(history.commits.len());
    let fresh =
        rebuild_name_index_from_durable_log(&history.persisted_through(history.commits.len()))
            .expect("the complete log reopens in a fresh installation");
    let head = live.head().unwrap();
    let names = live.name_keys();
    for name in &names {
        let admitted = admit_persisted_name(&fresh, name.subject_kind(), *name.digest())
            .unwrap_or_else(|refusal| panic!("{name:?} readmits: {refusal}"));
        assert_eq!(&admitted, name);
        for selection in history.selections() {
            assert_eq!(
                fresh.resolve(admitted, head, &selection),
                live.resolve(*name, head, &selection),
                "{name:?} under {selection:?}"
            );
        }
    }
    for subject in [
        PersistentNameSubjectKind::Entity,
        PersistentNameSubjectKind::Relation,
        PersistentNameSubjectKind::Group,
    ] {
        assert_eq!(
            admit_persisted_name(&fresh, subject, [0xab; 32]),
            Err(PersistentNameDurabilityError::UnknownPersistedName {
                subject,
                raw: [0xab; 32],
            })
        );
    }
    let entity = names
        .iter()
        .find(|name| matches!(name, PersistentNameKey::Entity(_)))
        .unwrap();
    assert!(matches!(
        admit_persisted_name(
            &fresh,
            PersistentNameSubjectKind::Relation,
            *entity.digest()
        ),
        Err(PersistentNameDurabilityError::UnknownPersistedName { .. })
    ));
}

#[test]
fn bare_kept_names_and_carriers_without_verified_births_refuse_typed() {
    let history = durable_naming_history();
    let refused = history.rebuild_mutated(|commits| {
        let restore = commits
            .iter_mut()
            .flat_map(|(batch, _)| batch.lineage_events.iter_mut())
            .find(|event| event.kind() == LineageEventKind::Restore)
            .expect("the history restores a name");
        restore.kept_name = Some(fabricated());
    });
    assert_eq!(
        refused,
        Err(unverified(
            fabricated(),
            PersistentNameReferenceRole::KeptName
        ))
    );

    let refused = history.rebuild_mutated(|commits| {
        let (batch, _) = commits
            .iter_mut()
            .find(|(batch, _)| {
                batch
                    .carriers
                    .contains_key(&PersistentNameKey::Entity(history.split_child))
            })
            .unwrap();
        batch
            .carriers
            .get_mut(&PersistentNameKey::Entity(history.split_child))
            .unwrap()
            .insert(fabricated());
    });
    assert_eq!(
        refused,
        Err(unverified(
            fabricated(),
            PersistentNameReferenceRole::Carrier
        ))
    );

    let refused = history.rebuild_mutated(|commits| {
        let (batch, _) = &mut commits[2];
        batch
            .carriers
            .insert(PersistentNameKey::Entity(fabricated()), Default::default());
    });
    assert_eq!(
        refused,
        Err(unverified(
            fabricated(),
            PersistentNameReferenceRole::CarrierKey
        ))
    );
}

#[test]
fn a_digest_consistent_birth_with_an_unborn_parent_refuses_typed() {
    let history = durable_naming_history();
    let holder = PersistentNameHolder::Entity(EntityId::new(PartitionId::main(), 900, 1));
    let request = face_birth(
        "occ.forged",
        "cut_piece",
        vec![fabricated()],
        "forged",
        match holder {
            PersistentNameHolder::Entity(entity) => entity,
            _ => unreachable!(),
        },
    );
    let minted = PersistentNameMintingAuthority::mint_commit_births(
        &cad_scheme_registry(),
        &BTreeMap::from([(fabricated(), holder)]),
        &Default::default(),
        &[request],
    )
    .expect("the minting authority mints over a parent it was told is live")
    .remove(0);
    let refused = history.rebuild_mutated(|commits| {
        let birth = commits[1]
            .0
            .lineage_events
            .iter_mut()
            .find(|event| event.name_birth().is_some())
            .unwrap();
        birth.name_birth = Some(minted);
    });
    assert_eq!(
        refused,
        Err(unverified(
            fabricated(),
            PersistentNameReferenceRole::BirthParent
        ))
    );
}
