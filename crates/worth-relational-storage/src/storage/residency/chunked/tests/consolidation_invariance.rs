//! Consolidation flattens layered history into a new member without moving
//! `root_semantic_merkle`; a diverging candidate or a tampered member is
//! refused by the semantic re-derivation (`M3`, consolidation half of `S2`).
use super::super::canonical_read::ChunkResidency;
use super::super::chunk_directory::RegionChunkDirectory;
use super::super::chunk_images::{ChunkHeadImage, ChunkHistoryImage, HistoryLayer, HistoryRow};
use super::super::consolidation::{
    consolidate_history_chunk, flatten_history_image, verify_bound_chunk, ChunkVerificationRefusal,
};
use super::super::member_frame::decode_member;
use super::super::member_frame::ChunkMemberKind;
use super::super::slot_range::{fixed_partition, SlotDomain};
use super::member_store::{accountant, edge, seal, version, MemberStore, SlotFixture, LAYOUT};
use crate::storage::residency::working_set::ResidencyPoolKind;

fn fixtures() -> Vec<SlotFixture> {
    (0..6)
        .map(|index| SlotFixture {
            slot: index * 9,
            head: Some(version(1 + (index % 2) as u32, index)),
            // Slot 36 spills; the others keep inline history to layer.
            history: (0..if index == 4 { 120 } else { 2 + index })
                .map(|v| version(5, index * 1_000 + v))
                .collect(),
            adjacency: (0..3).map(|e| edge(20, index * 10 + e)).collect(),
        })
        .collect()
}

/// Splits each row's inline versions across two layers, as a copy-on-write
/// edit that appended a newer layer would have stored them.
fn layered(image: &ChunkHistoryImage) -> ChunkHistoryImage {
    let (mut older, mut newer) = (HistoryLayer::default(), HistoryLayer::default());
    for row in &image.layers[0].rows {
        let split = row.versions.len() / 2;
        let (left, right) = row.versions.split_at(split);
        for (layer, versions) in [(&mut older, left), (&mut newer, right)] {
            if !versions.is_empty() {
                layer.rows.push(HistoryRow {
                    slot: row.slot,
                    versions: versions.to_vec(),
                });
            }
        }
    }
    ChunkHistoryImage {
        layers: vec![older, newer],
        ..image.clone()
    }
}

#[test]
fn consolidation_rebinds_history_and_keeps_the_semantic_root_bit_identical() {
    let store = MemberStore::default();
    let range = fixed_partition(64, LAYOUT.chunk_slots).next().unwrap();
    let sealed = seal(&store, SlotDomain::Entity, 64, range, &fixtures());
    let residency = ChunkResidency::new(
        &accountant(1 << 20, 1 << 20, 4096),
        ResidencyPoolKind::Historical,
    );
    let history = residency
        .history_image(&store, sealed.binding.history_locator())
        .unwrap();
    assert!(
        history.layers[0].rows.len() >= 3,
        "small slots keep inline history"
    );
    let layered = layered(&history)
        .encode(LAYOUT.max_chunk_image_bytes)
        .unwrap();
    store.replace(layered.locator, layered.bytes);
    // The layered world: same meaning, other physical history member.
    let layered_binding = sealed.binding.with_history_locator(layered.locator);
    verify_bound_chunk(&residency, &store, &sealed.semantic, &layered_binding).unwrap();
    let mut directory =
        RegionChunkDirectory::admit(LAYOUT, 64, 0, [(sealed.semantic.clone(), layered_binding)])
            .unwrap();
    let root_before = directory.root_semantic_merkle();

    let flat =
        flatten_history_image(&residency, &store, &mut &store, LAYOUT, &layered_binding).unwrap();
    assert_ne!(
        flat, layered.locator,
        "flattening changes the physical member"
    );
    let consolidated =
        consolidate_history_chunk(&residency, &store, &sealed.semantic, &layered_binding, flat)
            .unwrap();
    directory.install_consolidated(consolidated).unwrap();
    assert_eq!(directory.chunks()[0].binding().history_locator(), flat);
    assert_eq!(
        directory.root_semantic_merkle(),
        root_before,
        "root is locator-independent"
    );

    let mut lossy = history.flattened();
    lossy.layers[0].rows[2].versions.pop();
    let lossy = lossy.encode(LAYOUT.max_chunk_image_bytes).unwrap();
    store.replace(lossy.locator, lossy.bytes);
    assert!(matches!(
        consolidate_history_chunk(
            &residency,
            &store,
            &sealed.semantic,
            &layered_binding,
            lossy.locator
        ),
        Err(ChunkVerificationRefusal::CommitmentDiverged { .. })
    ));
    assert_eq!(
        directory.chunks()[0].binding().history_locator(),
        flat,
        "refusal installs nothing"
    );
}

#[test]
fn cold_verifier_refuses_a_member_whose_edge_set_differs_from_the_sealed_summary() {
    let store = MemberStore::default();
    let range = fixed_partition(64, LAYOUT.chunk_slots).next().unwrap();
    let sealed = seal(&store, SlotDomain::Entity, 64, range, &fixtures());
    let residency =
        ChunkResidency::new(&accountant(1 << 20, 1 << 20, 4096), ResidencyPoolKind::Head);
    verify_bound_chunk(&residency, &store, &sealed.semantic, &sealed.binding).unwrap();

    let head_bytes = store.bytes(sealed.binding.head_locator());
    let mut head: ChunkHeadImage = decode_member(
        ChunkMemberKind::Head,
        &head_bytes,
        sealed.binding.head_locator(),
    )
    .unwrap();
    head.slots[1].adjacency.pop();
    let tampered = head.encode(LAYOUT.max_chunk_image_bytes).unwrap();
    store.replace(tampered.locator, tampered.bytes);
    let omitted = super::super::locator_binding::ChunkLocatorBinding::new(
        sealed.binding.slot_range(),
        tampered.locator,
        sealed.binding.history_locator(),
        sealed.binding.continuation_directory_locator(),
    );
    assert!(matches!(
        verify_bound_chunk(&residency, &store, &sealed.semantic, &omitted),
        Err(ChunkVerificationRefusal::CommitmentDiverged { .. })
    ));
}

#[test]
fn cold_verifier_refuses_orphaned_continuation_directory_entries() {
    use super::super::continuation::{
        ContinuationEntries, ContinuationKey, ContinuationPage, ContinuationPageRef,
        ContinuationStream,
    };
    use super::super::member_frame::encode_member;
    use super::super::paged_directory::PagedDirectoryBuilder;
    use crate::storage::data::RelationalStorageUnavailableCause as Cause;

    let store = MemberStore::default();
    let range = fixed_partition(64, LAYOUT.chunk_slots).next().unwrap();
    // Slot 0 has only inline data, so continuation_directory_locator is None.
    let sealed = seal(
        &store,
        SlotDomain::Entity,
        64,
        range,
        &[SlotFixture {
            slot: 0,
            head: Some(version(1, 0)),
            history: vec![version(1, 1)],
            adjacency: Vec::new(),
        }],
    );
    assert_eq!(sealed.binding.continuation_directory_locator(), None);

    // Attach an orphaned continuation page for unoccupied slot 50.
    let orphan_key = ContinuationKey {
        slot: 50,
        stream: ContinuationStream::History,
        sequence: 0,
    };
    let orphan_page = ContinuationPage {
        key: orphan_key,
        entries: ContinuationEntries::History(vec![version(9, 99)]),
    };
    let encoded_page = encode_member(ChunkMemberKind::Continuation, &orphan_page, 4096).unwrap();
    store.replace(encoded_page.locator, encoded_page.bytes);
    let mut dir = PagedDirectoryBuilder::<ContinuationKey, ContinuationPageRef>::new(
        LAYOUT.directory_page_entries,
        4096,
    )
    .unwrap();
    dir.push(
        &mut &store,
        orphan_key,
        ContinuationPageRef {
            page: encoded_page.locator,
            entries: 1,
        },
    )
    .unwrap();
    let orphan_root = dir.finish(&mut &store).unwrap().unwrap();
    let residency = ChunkResidency::new(
        &accountant(1 << 20, 1 << 20, 4096),
        ResidencyPoolKind::Historical,
    );

    // Both an honest entry_count (1) and a forged entry_count (0) must be refused.
    for root in [
        orphan_root,
        super::super::paged_directory::PagedDirectoryRoot {
            entry_count: 0,
            ..orphan_root
        },
    ] {
        let tampered = super::super::locator_binding::ChunkLocatorBinding::new(
            sealed.binding.slot_range(),
            sealed.binding.head_locator(),
            sealed.binding.history_locator(),
            Some(root),
        );
        assert_eq!(
            verify_bound_chunk(&residency, &store, &sealed.semantic, &tampered),
            Err(ChunkVerificationRefusal::Member(Cause::Damaged))
        );
    }
}
