//! Paged `CHK24` record directories (Store M4.3d `G7`, `S2`): readmission
//! reads no page, kind scans fault only their own pages and evict under a
//! constant tight budget at `1x` and `10x` cardinality, the first edit
//! materializes exactly the recorded commitment, and tampered descriptors or
//! rosters refuse.
use super::membership::Membership;
use super::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::record_directory_pages::{
    encode_record_directory, MembershipImage, PagedRecordMembership, RecordDirectoryLeafFacts,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::member_io::{ChunkMemberSink, ChunkMemberSource};
use crate::storage::residency::working_set::{
    ResidencyBudget, ResidencyPoolKind, WorkingSetAccountant,
};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct Members(Mutex<HashMap<RelationalDurableMemberLocator, Vec<u8>>>);

impl ChunkMemberSink for &Members {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        self.0.lock().unwrap().insert(locator, bytes);
        Ok(())
    }
}

impl ChunkMemberSource for Members {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        let members = self.0.lock().unwrap();
        let bytes = members.get(&member).ok_or(Cause::MissingMember)?;
        let tail = &bytes[offset as usize..];
        let count = tail.len().min(target.len());
        target[..count].copy_from_slice(&tail[..count]);
        Ok(count)
    }
}

/// Sorted keys: `count` members of each kind on distinct slots.
fn keys(kinds: &[(u32, u64)]) -> Vec<u128> {
    let mut slot = 0;
    let mut keys = Vec::new();
    for &(kind, count) in kinds {
        for _ in 0..count {
            keys.push(((kind as u128) << 96) | ((slot as u128) << 32) | 1);
            slot += 1;
        }
    }
    keys.sort_unstable();
    keys
}

/// The production resident commitment, built independently of paging.
fn commitment(keys: &[u128]) -> [u8; 32] {
    crate::storage::residency::resident_history::inspect_reconstruction_fixture(|lane, _| {
        let Membership::Resident(resident) = Membership::cold(lane, keys.to_vec()).unwrap() else {
            unreachable!("cold construction is resident")
        };
        resident.digest()
    })
}

fn tight() -> Arc<WorkingSetAccountant> {
    WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: 48 * 1024,
        historical_working_set_bytes: 48 * 1024,
        max_in_flight_fill_bytes: 1 << 20,
        streaming_window_bytes: 4096,
    })
}

struct Fixture {
    members: Arc<Members>,
    descriptor: PagedRecordDirectory,
    entities: Vec<u128>,
}

fn fixture(scale: u64) -> Fixture {
    let entities = keys(&[(1, 1_000 * scale), (2, 990 * scale), (3, 5), (4, 5 * scale)]);
    let relations = keys(&[(7, 300 * scale)]);
    let members = Arc::new(Members::default());
    let descriptor = encode_record_directory(
        MembershipImage {
            keys: &entities,
            digest: commitment(&entities),
        },
        MembershipImage {
            keys: &relations,
            digest: commitment(&relations),
        },
        &mut &*members,
    )
    .unwrap();
    Fixture {
        members,
        descriptor,
        entities,
    }
}

fn open(
    fixture: &Fixture,
    roster: Vec<RelationalDurableMemberLocator>,
) -> Arc<RegionRecordDirectory> {
    let pages = RecordDirectoryPages::new(
        roster,
        fixture.members.clone(),
        tight(),
        ResidencyPoolKind::Head,
    );
    RegionRecordDirectory::from_paged(
        PartitionId::main(),
        Arc::new(fixture.descriptor.clone()),
        pages,
    )
}

fn faults(directory: &RegionRecordDirectory) -> u64 {
    directory.page_observation().unwrap().faults
}

#[test]
fn reopen_faults_no_page_and_kind_scans_fault_only_their_pages_at_1x_and_10x() {
    for scale in [1, 10] {
        let fixture = fixture(scale);
        let directory = open(&fixture, fixture.descriptor.pages.clone());
        assert_eq!(faults(&directory), 0, "readmission reads no page");
        assert!(directory.has_entity_kind(KindId(1)) && directory.has_relation_kind(KindId(7)));
        assert!(!directory.has_entity_kind(KindId(9)));
        let _commitment = directory.digest();
        assert_eq!(directory.entity_ids(KindId(9)).count(), 0, "absent kind");
        assert_eq!(faults(&directory), 0, "presence and absence read no page");

        let small = directory
            .entity_ids(KindId(3))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(small.len(), 5);
        assert!(faults(&directory) <= 4, "one root-to-leaf path at {scale}x");

        let all = directory
            .entity_ids(KindId(1))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let expected = fixture
            .entities
            .iter()
            .filter(|key| (**key >> 96) == 1)
            .map(|key| (key >> 32) as u64)
            .collect::<Vec<_>>();
        assert_eq!(
            all.iter()
                .map(|id| id.slot_index() as u64)
                .collect::<Vec<_>>(),
            expected
        );
        let observation = directory.page_observation().unwrap();
        let pages = fixture.descriptor.pages.len() as u64;
        assert!(
            observation.faults < pages,
            "kind scan skips other kinds' pages"
        );
        if scale == 10 {
            assert!(
                observation.evictions > 0,
                "pages evict under the constant budget"
            );
        }
    }
}

#[test]
fn first_edit_materializes_the_recorded_commitment_and_keeps_deltas_exact() {
    let fixture = fixture(1);
    let directory = open(&fixture, fixture.descriptor.pages.clone());
    let mut next = (*directory).clone();
    let added = (3_u128 << 96) | (90_000_u128 << 32) | 1;
    next.entities
        .update(added, true, &mut |_, _| Ok(()))
        .unwrap();
    let Membership::Resident(resident) = &next.entities else {
        panic!("an edited domain is resident")
    };
    let mut expected = fixture.entities.clone();
    expected.push(added);
    expected.sort_unstable();
    assert_eq!(resident.digest(), commitment(&expected));
    assert_eq!(
        next.changed_entity_kinds(&directory).collect::<Vec<_>>(),
        [KindId(3)],
        "the delta against the shared materialized image is structural"
    );
    assert_eq!(next.changed_relation_kinds(&directory).count(), 0);
}

#[test]
fn materialization_refuses_a_descriptor_whose_commitment_disagrees() {
    let mut fixture = fixture(1);
    fixture.descriptor.entities.digest[0] ^= 1;
    let directory = open(&fixture, fixture.descriptor.pages.clone());
    let mut next = (*directory).clone();
    let refused = next.entities.update(0, true, &mut |_, _| Ok(()));
    assert!(matches!(refused, Err(DirectoryError::Corrupt)));
}

#[test]
fn an_interior_page_naming_an_unlisted_or_missing_page_refuses() {
    let fixture = fixture(1);
    let root = fixture.descriptor.entities.root.unwrap().root_page;
    let mut roster = fixture.descriptor.pages.clone();
    let unlisted = *roster.iter().find(|page| **page != root).unwrap();
    roster.retain(|page| *page != unlisted);
    let directory = open(&fixture, roster);
    let refused = (1..=4)
        .flat_map(|kind| directory.entity_ids(KindId(kind)).collect::<Vec<_>>())
        .chain(
            directory
                .relation_ids(KindId(7))
                .map(|id| id.map(|_| EntityId::new(PartitionId::main(), 0, 0))),
        )
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert!(
        !refused.is_empty()
            && refused
                .iter()
                .all(|failure| failure.cause == Cause::Damaged)
    );

    fixture.members.0.lock().unwrap().remove(&unlisted);
    let directory = open(&fixture, fixture.descriptor.pages.clone());
    let missing = (1..=4)
        .flat_map(|kind| directory.entity_ids(KindId(kind)).collect::<Vec<_>>())
        .chain(
            directory
                .relation_ids(KindId(7))
                .map(|id| id.map(|_| EntityId::new(PartitionId::main(), 0, 0))),
        )
        .filter_map(Result::err)
        .collect::<Vec<_>>();
    assert!(missing
        .iter()
        .any(|failure| failure.cause == Cause::MissingMember));
}

#[test]
fn tampered_descriptors_refuse_without_reading_any_page() {
    let fixture = fixture(1);
    let live = |descriptor: &PagedRecordDirectory| RecordDirectoryLeafFacts {
        live_entities: descriptor
            .entities
            .root
            .map_or(0, |root| root.entry_count as usize),
        live_relations: descriptor
            .relations
            .root
            .map_or(0, |root| root.entry_count as usize),
        entity_slots: 1 << 20,
        relation_slots: 1 << 20,
    };
    let valid = fixture.descriptor.clone();
    assert_eq!(valid.validate(live(&valid)), Ok(()));
    let tampers: [(&str, fn(&mut PagedRecordDirectory)); 9] = [
        ("unsorted roster", |d| d.pages.swap(0, 1)),
        ("duplicate roster", |d| d.pages[1] = d.pages[0]),
        ("oversized page", |d| {
            d.pages.last_mut().unwrap().length = 1 << 20
        }),
        ("orphan root", |d| {
            let root = d.entities.root.unwrap().root_page;
            d.pages.retain(|page| *page != root);
        }),
        ("kinds unsorted", |d| d.entities.kinds.swap(0, 1)),
        ("kinds missing", |d| d.relations.kinds.clear()),
        ("empty with kinds", |d| {
            d.relations.root = None;
            d.relations.kinds = vec![KindId(7)];
        }),
        ("empty with digest", |d| {
            d.relations.root = None;
            d.relations.kinds.clear();
            d.relations.digest = [9; 32];
        }),
        ("roster without roots", |d| {
            d.entities = PagedRecordMembership::empty();
            d.relations = PagedRecordMembership::empty();
        }),
    ];
    for (name, tamper) in tampers {
        let mut tampered = valid.clone();
        tamper(&mut tampered);
        let facts = live(&tampered);
        assert!(tampered.validate(facts).is_err(), "{name} must refuse");
    }
    let mut facts = live(&valid);
    facts.live_entities += 1;
    assert!(
        valid.validate(facts).is_err(),
        "count disagrees with live records"
    );
    let mut facts = live(&valid);
    facts.entity_slots = 0;
    assert!(
        valid.validate(facts).is_err(),
        "entries claimed in an empty arena"
    );
    let mut zero = valid.clone();
    zero.entities.root.as_mut().unwrap().entry_count = 0;
    assert!(
        zero.validate(live(&valid)).is_err(),
        "a present root has entries"
    );
    let mut overflow = valid.clone();
    overflow.entities.root.as_mut().unwrap().entry_count = u64::MAX;
    overflow.relations.root.as_mut().unwrap().entry_count = u64::MAX;
    let facts = RecordDirectoryLeafFacts {
        live_entities: usize::MAX,
        live_relations: usize::MAX,
        entity_slots: usize::MAX,
        relation_slots: usize::MAX,
    };
    assert!(overflow.validate(facts).is_err(), "entry counts overflow");
}

#[test]
fn aspect_only_edit_materializes_paged_domains_and_understated_count_refuses() {
    let f = fixture(1);
    let paged = open(&f, f.descriptor.pages.clone());
    assert!(paged.has_paged_domain());
    let updated =
        maintenance::finalize_edit(&paged, (*paged).clone(), false, &mut |_, _| Ok(())).unwrap();
    assert!(!updated.has_paged_domain());
    // Dropping all backing page members must not break scans of the updated directory.
    f.members.0.lock().unwrap().clear();
    assert_eq!(updated.entity_ids(KindId(3)).count(), 5);
    assert_eq!(updated.relation_ids(KindId(7)).count(), 300);

    // P2: an understated `entry_count` refuses before building ResidentMembership.
    let mut f2 = fixture(1);
    f2.descriptor.entities.root.as_mut().unwrap().entry_count = 1;
    let tampered = open(&f2, f2.descriptor.pages.clone());
    let result =
        maintenance::finalize_edit(&tampered, (*tampered).clone(), false, &mut |_, _| Ok(()));
    assert!(matches!(result, Err(DirectoryError::Corrupt)));
}

#[test]
fn only_cold_domains_route_scans_through_held_heads_and_exhausted_ids_drop_pages() {
    let fixture = fixture(1);
    let directory = open(&fixture, fixture.descriptor.pages.clone());
    assert!(directory.entity_scan_needs_cold_pages() && directory.relation_scan_needs_cold_pages());
    let mut next = (*directory).clone();
    let added = (3_u128 << 96) | (90_000_u128 << 32) | 1;
    next.entities
        .update(added, true, &mut |_, _| Ok(()))
        .unwrap();
    assert!(
        !next.entity_scan_needs_cold_pages(),
        "an edited domain scans its map"
    );
    assert!(!RegionRecordDirectory::empty(PartitionId::main()).entity_scan_needs_cold_pages());
    let Membership::Paged(paged) = &directory.relations else {
        panic!("a readmitted domain is paged")
    };
    let mut ids =
        paged.ids::<crate::identity::data::RelationDomain>(KindId(7), PartitionId::main());
    assert_eq!(ids.by_ref().filter(Result::is_ok).count(), 300);
    assert!(
        !ids.holds_cursor(),
        "an exhausted iterator releases its pages"
    );
    let failing = open(&fixture, Vec::new());
    let Membership::Paged(paged) = &failing.relations else {
        panic!("a readmitted domain is paged")
    };
    let mut err_ids =
        paged.ids::<crate::identity::data::RelationDomain>(KindId(7), PartitionId::main());
    assert!(err_ids.next().unwrap().is_err());
    assert!(
        !err_ids.holds_cursor(),
        "a failing iterator releases its pages"
    );
}
