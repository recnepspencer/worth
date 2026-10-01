use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    durable_artifact_checksum, PersistedRecordIdentity, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
};

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn entry(object: u8) -> ReleaseCustodyHeadEntryV1 {
    let record = |ordinal| PersistedRecordIdentity::new([object; 16], ordinal).unwrap();
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
        record(1),
        [1; 32],
        record(2),
        [2; 32],
        record(3),
        [3; 32],
        [4; 32],
        None,
        1,
        1,
        false,
    )
    .unwrap()
}

fn limits() -> ReleaseCustodyHeadWalkLimitsV1 {
    ReleaseCustodyHeadWalkLimitsV1::new(8, 8, 8 * 16_384, 128 * 1024, 4).unwrap()
}

#[test]
fn rooted_walk_streams_all_entries_and_rejects_substituted_valid_node() {
    let left = ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![entry(1)], format()).unwrap();
    let right = ReleaseCustodyHeadBlockV1::leaf(9, 2, 2, vec![entry(2)], format()).unwrap();
    let branch = ReleaseCustodyHeadBlockV1::branch(
        9,
        2,
        3,
        1,
        vec![left.reference(format()), right.reference(format())],
        format(),
    )
    .unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(branch.reference(format())))
        .next_release_custody_head_block(4)
        .admit()
        .unwrap();
    let mut visited = Vec::new();
    let result = walk_release_custody_head(
        &root,
        format(),
        limits(),
        |reference, _limit| {
            Ok::<_, ()>(match reference.block() {
                1 => left.encode(format()),
                2 => right.encode(format()),
                3 => branch.encode(format()),
                _ => panic!("unexpected rooted block"),
            })
        },
        |head| {
            visited.push(head.key());
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert_eq!(visited, vec![entry(1).key(), entry(2).key()]);
    assert_eq!(result.entry_count(), 2);
    assert_eq!(result.node_count(), 3);
    assert_ne!(result.roster_digest(), [0; 32]);

    let denial = walk_release_custody_head(
        &root,
        format(),
        limits(),
        |reference, _| {
            Ok::<_, ()>(match reference.block() {
                1 => right.encode(format()),
                2 => right.encode(format()),
                3 => branch.encode(format()),
                _ => panic!("unexpected rooted block"),
            })
        },
        |_| Ok::<_, ()>(()),
    )
    .unwrap_err();
    assert!(matches!(
        denial,
        ReleaseCustodyHeadWalkDenial::Format(
            ReleaseCustodyHeadDenial::Digest
                | ReleaseCustodyHeadDenial::Identity
                | ReleaseCustodyHeadDenial::Reference
        )
    ));
}

#[test]
fn rooted_walk_enforces_shared_entry_budget_before_visiting_excess() {
    let block =
        ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![entry(1), entry(2)], format()).unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(block.reference(format())))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();
    let narrow = ReleaseCustodyHeadWalkLimitsV1::new(2, 1, 32_768, 128 * 1024, 2).unwrap();
    let mut visited = 0;
    let denial = walk_release_custody_head(
        &root,
        format(),
        narrow,
        |_, _| Ok::<_, ()>(block.encode(format())),
        |_| {
            visited += 1;
            Ok::<_, ()>(())
        },
    )
    .unwrap_err();
    assert_eq!(
        denial,
        ReleaseCustodyHeadWalkDenial::Format(ReleaseCustodyHeadDenial::Capacity)
    );
    assert_eq!(visited, 1);
}

#[test]
fn rooted_walk_denies_insufficient_first_node_residency_without_reading() {
    let block = ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![entry(1)], format()).unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(block.reference(format())))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();
    let minimum =
        ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format(), 8).unwrap();
    let narrow = limits().with_max_resident_bytes(minimum - 1).unwrap();
    let mut reads = 0;
    let denial = walk_release_custody_head(
        &root,
        format(),
        narrow,
        |_, _| {
            reads += 1;
            Ok::<_, ()>(block.encode(format()))
        },
        |_| Ok::<_, ()>(()),
    )
    .unwrap_err();
    assert_eq!(
        denial,
        ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded {
            required: minimum,
            admitted: minimum - 1,
        }
    );
    assert_eq!(reads, 0);
}

#[test]
fn rooted_walk_denies_branch_stack_growth_before_next_read() {
    let leaves: Vec<_> = (1..=20)
        .map(|index| {
            ReleaseCustodyHeadBlockV1::leaf(9, 2, index, vec![entry(index as u8)], format())
                .unwrap()
        })
        .collect();
    let branch = ReleaseCustodyHeadBlockV1::branch(
        9,
        2,
        21,
        1,
        leaves.iter().map(|leaf| leaf.reference(format())).collect(),
        format(),
    )
    .unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(branch.reference(format())))
        .next_release_custody_head_block(22)
        .admit()
        .unwrap();
    let first_node =
        ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format(), 32).unwrap();
    let stack_slot = std::mem::size_of::<(ReleaseCustodyHeadBlockReferenceV1, u16)>() as u64;
    let ceiling = first_node + 4 * stack_slot;
    let bound = ReleaseCustodyHeadWalkLimitsV1::new(32, 20, 32 * 16_384, ceiling, 2).unwrap();
    let mut reads = 0;
    let denial = walk_release_custody_head(
        &root,
        format(),
        bound,
        |_, _| {
            reads += 1;
            Ok::<_, ()>(branch.encode(format()))
        },
        |_| Ok::<_, ()>(()),
    )
    .unwrap_err();
    assert!(matches!(
        denial,
        ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded { required, admitted }
            if required > admitted
    ));
    assert_eq!(reads, 1);
}

#[test]
fn multilevel_branch_denies_old_and_new_stack_overlap_before_leaf_read() {
    let leaves: Vec<_> = (1..=151)
        .map(|index| {
            ReleaseCustodyHeadBlockV1::leaf(9, 2, index, vec![entry(index as u8)], format())
                .unwrap()
        })
        .collect();
    let first = ReleaseCustodyHeadBlockV1::branch(
        9,
        2,
        152,
        1,
        leaves[..150]
            .iter()
            .map(|leaf| leaf.reference(format()))
            .collect(),
        format(),
    )
    .unwrap();
    let second = ReleaseCustodyHeadBlockV1::branch(
        9,
        2,
        153,
        1,
        vec![leaves[150].reference(format())],
        format(),
    )
    .unwrap();
    let top = ReleaseCustodyHeadBlockV1::branch(
        9,
        2,
        154,
        2,
        vec![first.reference(format()), second.reference(format())],
        format(),
    )
    .unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(top.reference(format())))
        .next_release_custody_head_block(155)
        .admit()
        .unwrap();
    let stack_slot = std::mem::size_of::<(ReleaseCustodyHeadBlockReferenceV1, u16)>() as u64;
    let seen = super::seen::requested_bytes(154).unwrap();
    let top_frame = top.encode(format()).len() as u64;
    let first_frame = first.encode(format()).len() as u64;
    // The completed new stack fits alongside the frame. Only the old+new
    // allocation overlap during growth exceeds the admitted ceiling.
    let ceiling = seen + 152 * stack_slot + first_frame;
    assert!(seen + 2 * stack_slot + u64::from(format().page_size().bytes()) <= ceiling);
    assert!(seen + 3 * stack_slot + top_frame <= ceiling);
    assert!(seen + 151 * stack_slot + first_frame <= ceiling);
    assert!(seen + 153 * stack_slot + first_frame > ceiling);
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(154, 151, 154 * 16_384, ceiling, 3).unwrap();
    let mut reads = Vec::new();
    let denial = walk_release_custody_head(
        &root,
        format(),
        limits,
        |reference, _| {
            reads.push(reference.block());
            Ok::<_, ()>(match reference.block() {
                154 => top.encode(format()),
                152 => first.encode(format()),
                _ => panic!("stack growth must deny before another rooted read"),
            })
        },
        |_| Ok::<_, ()>(()),
    )
    .unwrap_err();
    assert!(matches!(
        denial,
        ReleaseCustodyHeadWalkDenial::ResidentBoundExceeded { required, admitted }
            if required > admitted
    ));
    assert_eq!(reads, vec![154, 152]);
}

#[test]
fn malformed_late_entry_denies_before_node_or_entry_callbacks() {
    let block =
        ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![entry(1), entry(2)], format()).unwrap();
    let original = block.reference(format());
    let mut frame = block.encode(format());
    frame[48 + 40 + ReleaseCustodyHeadEntryV1::ENCODED_BYTES + 305] = 1;
    let mut checksum_input = frame[..44].to_vec();
    checksum_input.extend_from_slice(&frame[48..]);
    frame[44..48].copy_from_slice(&durable_artifact_checksum(&checksum_input).to_le_bytes());
    let reference = ReleaseCustodyHeadBlockReferenceV1::new(
        original.generation(),
        original.block(),
        original.level(),
        original.first(),
        original.last(),
        Sha256::digest(&frame).into(),
    )
    .unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(reference))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();

    struct RejectCallbacks(Vec<u8>);
    impl ReleaseCustodyHeadWalkPort for RejectCallbacks {
        type Error = ();
        fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Self::Error> {
            Ok(Vec::with_capacity(count))
        }
        fn grow_vec<T>(
            &mut self,
            values: &mut Vec<T>,
            additional: usize,
        ) -> Result<(), Self::Error> {
            values.reserve(additional);
            Ok(())
        }
        fn discard_vec<T>(&mut self, _: Vec<T>) {}
        fn read_node(
            &mut self,
            _: ReleaseCustodyHeadBlockReferenceV1,
            _: u64,
        ) -> Result<Vec<u8>, Self::Error> {
            Ok(self.0.clone())
        }
        fn visit_node(
            &mut self,
            _: ReleaseCustodyHeadBlockReferenceV1,
            _: &[u8],
        ) -> Result<(), Self::Error> {
            panic!("malformed node was exposed")
        }
        fn visit_entry(&mut self, _: ReleaseCustodyHeadEntryV1) -> Result<(), Self::Error> {
            panic!("malformed entry was exposed")
        }
    }
    assert_eq!(
        walk_release_custody_head_with_port(&root, format(), limits(), &mut RejectCallbacks(frame)),
        Err(ReleaseCustodyHeadWalkDenial::Format(
            ReleaseCustodyHeadDenial::Malformed
        ))
    );
}

#[test]
fn legacy_walk_preserves_typed_allocation_failure_before_read() {
    let block = ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![entry(1)], format()).unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(block.reference(format())))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();
    let slot = std::mem::size_of::<Option<(u64, u64)>>() as u64;
    let max_nodes = (isize::MAX as u64 / slot) / 2 + 1;
    let limits = ReleaseCustodyHeadWalkLimitsV1::new(max_nodes, 1, 16_384, u64::MAX, 1).unwrap();
    let mut reads = 0;
    let denial = walk_release_custody_head(
        &root,
        format(),
        limits,
        |_, _| {
            reads += 1;
            Ok::<_, ()>(block.encode(format()))
        },
        |_| Ok::<_, ()>(()),
    )
    .unwrap_err();
    assert!(matches!(
        denial,
        ReleaseCustodyHeadWalkDenial::Allocation { requested, cause: _ }
            if requested > isize::MAX as u64
    ));
    assert_eq!(reads, 0);
}
