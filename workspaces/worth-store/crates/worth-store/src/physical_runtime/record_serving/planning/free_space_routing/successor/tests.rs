use super::*;
use crate::physical_runtime::record_serving::RecordAppendDenial;
use std::cell::Cell;
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

struct CanonicalBlocks {
    blocks: Vec<(FreeSpaceBlockReference, PhysicalFreeSpaceMembershipBlock)>,
    reads: Cell<usize>,
}

impl CanonicalBlocks {
    fn insert(
        &mut self,
        block: PhysicalFreeSpaceMembershipBlock,
        format: PhysicalRecordFormatDeclaration,
    ) -> FreeSpaceBlockReference {
        let frame = block.encode(format);
        let reference = block.reference(durable_artifact_checksum(&frame));
        self.blocks.push((reference, block));
        reference
    }
}

impl FreeSpaceBlockSource for CanonicalBlocks {
    fn read_block(
        &self,
        reference: FreeSpaceBlockReference,
        _discovery: &mut ManifestDiscoveryCounterSnapshot,
    ) -> Result<PhysicalFreeSpaceMembershipBlock, ManifestLookupFailure> {
        self.reads.set(self.reads.get() + 1);
        self.blocks
            .iter()
            .find(|(candidate, _)| *candidate == reference)
            .map(|(_, block)| block.clone())
            .ok_or(ManifestLookupFailure::Damaged)
    }
}

fn entry(owner: u64) -> RecordFreeSpaceManifestEntry {
    RecordFreeSpaceManifestEntry::inline_frontier(owner, 1, 1, 1).unwrap()
}

fn source_with_five_entries(
    first_leaf_entries: u64,
    format: PhysicalRecordFormatDeclaration,
) -> (CanonicalBlocks, DurableFreeSpaceManifestHeader) {
    let mut source = CanonicalBlocks {
        blocks: Vec::new(),
        reads: Cell::new(0),
    };
    let first = source.insert(
        PhysicalFreeSpaceMembershipBlock::leaf(
            7,
            1,
            1,
            (1..=first_leaf_entries).map(entry).collect(),
            4,
        )
        .unwrap(),
        format,
    );
    let second = source.insert(
        PhysicalFreeSpaceMembershipBlock::leaf(
            7,
            1,
            2,
            ((first_leaf_entries + 1)..=5).map(entry).collect(),
            4,
        )
        .unwrap(),
        format,
    );
    let root = source.insert(
        PhysicalFreeSpaceMembershipBlock::branch(7, 1, 3, 1, vec![first, second], 4).unwrap(),
        format,
    );
    let header = DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        1,
        7,
        4,
        1,
        5,
        2,
        2,
        2,
        2,
        Some(1),
        131_072,
        4096,
        4,
        Some(root),
    )
    .unwrap();
    (source, header)
}

fn source_with_seventeen_entries(
    format: PhysicalRecordFormatDeclaration,
) -> (CanonicalBlocks, DurableFreeSpaceManifestHeader) {
    let mut source = CanonicalBlocks {
        blocks: Vec::new(),
        reads: Cell::new(0),
    };
    let leaves = (0..5)
        .map(|index| {
            let first = index * 4 + 1;
            let last = (first + 3).min(17);
            source.insert(
                PhysicalFreeSpaceMembershipBlock::leaf(
                    7,
                    1,
                    index + 1,
                    (first..=last).map(entry).collect(),
                    4,
                )
                .unwrap(),
                format,
            )
        })
        .collect::<Vec<_>>();
    let first_branch = source.insert(
        PhysicalFreeSpaceMembershipBlock::branch(7, 1, 6, 1, leaves[..4].to_vec(), 4).unwrap(),
        format,
    );
    let second_branch = source.insert(
        PhysicalFreeSpaceMembershipBlock::branch(7, 1, 7, 1, vec![leaves[4]], 4).unwrap(),
        format,
    );
    let root = source.insert(
        PhysicalFreeSpaceMembershipBlock::branch(7, 1, 8, 2, vec![first_branch, second_branch], 4)
            .unwrap(),
        format,
    );
    let header = DurableFreeSpaceManifestHeader::new_with_tier_epoch(
        1,
        7,
        4,
        1,
        17,
        2,
        2,
        2,
        2,
        Some(1),
        131_072,
        4096,
        9,
        Some(root),
    )
    .unwrap();
    (source, header)
}

#[test]
fn exhausting_one_of_five_entries_repacks_capacity_four_tree_to_leaf() {
    let declaration = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(declaration);
    // Both a vanished first leaf and two surviving leaves must repack when
    // the remaining four entries fit the canonical root leaf.
    for first_leaf_entries in [1, 2] {
        let (source, current) = source_with_five_entries(first_leaf_entries, declaration);
        let exhausted = FreeSpaceKey::inline(first_leaf_entries).unwrap();
        let plan = plan_with_source(
            &source,
            format,
            &current,
            FreeSpaceSuccessorRequest {
                generation: 2,
                node_capacity: 4,
                segment_page_capacity: current.segment_page_capacity(),
                next_segment: current.next_segment(),
                next_page: current.next_page(),
                next_extent: current.next_extent(),
                next_arena: current.next_arena(),
                updates: BTreeMap::from([(exhausted, FreeSpaceUpdate::Exhausted)]),
            },
            super::repack::local_peak_bound(4, 4, 1, 4, u64::from(declaration.page_size().bytes()))
                .unwrap()
                .1,
        )
        .expect("a genuine one-entry exhaustion leaves a canonical four-entry successor");
        assert_eq!(plan.header.entry_count(), 4);
        assert_eq!(plan.header.tree_identity(), current.tree_identity());
        assert_eq!(plan.header.tier_epoch_start(), current.tier_epoch_start());
        assert_eq!(plan.header.next_segment(), current.next_segment());
        assert_eq!(plan.header.next_page(), current.next_page());
        assert_eq!(plan.header.next_extent(), current.next_extent());
        assert_eq!(plan.header.next_arena(), current.next_arena());

        let root = plan.header.root().unwrap();
        assert_eq!(root.level(), 0, "four entries must occupy one root leaf");
        let expected_artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: root.generation(),
            block: root.block(),
        };
        let frame = plan
            .blocks
            .iter()
            .find(|(artifact, _)| *artifact == expected_artifact)
            .map(|(_, frame)| frame.clone())
            .or_else(|| {
                source
                    .blocks
                    .iter()
                    .find(|(reference, _)| *reference == root)
                    .map(|(_, block)| block.encode(declaration))
            })
            .expect("the successor root is staged or reuses an exact canonical source block");
        assert_eq!(root.checksum(), durable_artifact_checksum(&frame));
        let (block, decoded_format) = PhysicalFreeSpaceMembershipBlock::decode(&frame, 4).unwrap();
        assert_eq!(decoded_format, declaration);
        let PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } = block else {
            panic!("the canonical successor root must be a leaf")
        };
        let owners = entries
            .iter()
            .map(|entry| entry.owner())
            .collect::<Vec<_>>();
        let expected = (1..=5)
            .filter(|owner| *owner != first_leaf_entries)
            .collect::<Vec<_>>();
        assert_eq!(owners, expected);
    }
}

#[test]
fn streaming_repack_cost_denies_before_second_source_walk() {
    let declaration = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let (source, current) = source_with_five_entries(2, declaration);
    let required =
        super::repack::local_peak_bound(4, 4, 1, 4, u64::from(declaration.page_size().bytes()))
            .unwrap()
            .1;
    let denial = plan_with_source(
        &source,
        AdmittedPhysicalRecordFormat::admit(declaration),
        &current,
        FreeSpaceSuccessorRequest {
            generation: 2,
            node_capacity: 4,
            segment_page_capacity: current.segment_page_capacity(),
            next_segment: current.next_segment(),
            next_page: current.next_page(),
            next_extent: current.next_extent(),
            next_arena: current.next_arena(),
            updates: BTreeMap::from([(
                FreeSpaceKey::inline(2).unwrap(),
                FreeSpaceUpdate::Exhausted,
            )]),
        },
        required - 1,
    )
    .err()
    .expect("insufficient local repack admission must deny");
    assert!(matches!(
        denial,
        RecordAppendError::Denied(RecordAppendDenial::ResidencyUnavailable(_))
    ));
    assert_eq!(source.reads.get(), 2, "the fallback made no source read");
}

#[test]
fn height_two_seventeen_to_sixteen_repacks_full_ancestry() {
    let declaration = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let (source, current) = source_with_seventeen_entries(declaration);
    let plan = plan_with_source(
        &source,
        AdmittedPhysicalRecordFormat::admit(declaration),
        &current,
        FreeSpaceSuccessorRequest {
            generation: 2,
            node_capacity: 4,
            segment_page_capacity: current.segment_page_capacity(),
            next_segment: current.next_segment(),
            next_page: current.next_page(),
            next_extent: current.next_extent(),
            next_arena: current.next_arena(),
            updates: BTreeMap::from([(
                FreeSpaceKey::inline(1).unwrap(),
                FreeSpaceUpdate::Exhausted,
            )]),
        },
        super::repack::local_peak_bound(16, 4, 2, 4, u64::from(declaration.page_size().bytes()))
            .unwrap()
            .1,
    )
    .expect("one exhaustion must repack the two-level source into four leaves");

    assert_eq!(plan.header.entry_count(), 16);
    assert_eq!(plan.header.tree_identity(), current.tree_identity());
    assert_eq!(plan.header.tier_epoch_start(), current.tier_epoch_start());
    assert_eq!(plan.header.next_segment(), current.next_segment());
    assert_eq!(plan.header.next_page(), current.next_page());
    assert_eq!(plan.header.next_extent(), current.next_extent());
    assert_eq!(plan.header.next_arena(), current.next_arena());
    assert_eq!(plan.header.next_block(), current.next_block() + 5);
    assert_eq!(plan.blocks.len(), 5, "four packed leaves and one branch");
    for (index, (artifact, _)) in plan.blocks.iter().enumerate() {
        assert_eq!(
            *artifact,
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: 2,
                block: current.next_block() + u64::try_from(index).unwrap(),
            }
        );
    }
    assert!(
        source.reads.get() > source.blocks.len(),
        "fallback reread the source"
    );

    let root = plan.header.root().unwrap();
    assert_eq!(root.level(), 1);
    assert_eq!(root.generation(), 2);
    let staged_frame = |reference: FreeSpaceBlockReference| {
        plan.blocks
            .iter()
            .find(|(artifact, _)| {
                *artifact
                    == RecordArtifactFile::FreeSpaceMembershipBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
            })
            .map(|(_, frame)| frame.as_slice())
            .expect("every packed block is staged at the source frontier")
    };
    let root_frame = staged_frame(root);
    assert_eq!(root.checksum(), durable_artifact_checksum(root_frame));
    let (root_block, root_format) =
        PhysicalFreeSpaceMembershipBlock::decode(root_frame, 4).unwrap();
    assert_eq!(root_format, declaration);
    assert_eq!(
        root_block.reference(durable_artifact_checksum(root_frame)),
        root
    );
    let PhysicalFreeSpaceMembershipBlock::Branch { children, .. } = root_block else {
        panic!("the packed root must be a branch")
    };
    assert_eq!(children.len(), 4);
    let mut owners = Vec::new();
    for child in children {
        assert_eq!(child.level(), 0);
        let frame = staged_frame(child);
        assert_eq!(child.checksum(), durable_artifact_checksum(frame));
        let (block, format) = PhysicalFreeSpaceMembershipBlock::decode(frame, 4).unwrap();
        assert_eq!(format, declaration);
        assert_eq!(block.reference(durable_artifact_checksum(frame)), child);
        let PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } = block else {
            panic!("each packed child must be a leaf")
        };
        owners.extend(entries.iter().map(|entry| entry.owner()));
    }
    assert_eq!(owners, (2..=17).collect::<Vec<_>>());
}
