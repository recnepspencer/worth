use super::{BTreeNodeCellV1 as Cell, BTreeNodeDenial as Denial, BTreeNodeV1 as Node};
use crate::record_framing::crc32c;
use crate::PersistedRecordIdentity;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

fn leaf() -> Node {
    Node::leaf(
        4,
        (0_u16..300)
            .map(|index| Cell::leaf(index.to_be_bytes().to_vec(), vec![index as u8; 24]))
            .collect(),
        None,
        Some(record(2)),
    )
    .unwrap()
}

fn reseal(bytes: &mut [u8]) {
    let checksum = crc32c::checksum(&[&bytes[..96], &bytes[100..]]);
    bytes[96..100].copy_from_slice(&checksum.to_le_bytes());
}

#[test]
fn n_ary_leaf_round_trips_with_300_ordered_cells_inside_minimum_page() {
    let node = leaf();
    let bytes = node.encode(16_272).unwrap();
    assert!(bytes.len() < 16_272);
    assert_eq!(Node::decode(&bytes), Ok(node));
}

#[test]
fn interior_children_and_sibling_hints_remain_distinct() {
    let node = Node::interior(
        4,
        2,
        record(4),
        vec![
            Cell::interior(b"alpha".to_vec(), record(5)),
            Cell::interior(b"omega".to_vec(), record(6)),
        ],
        Some(record(2)),
        Some(record(7)),
    )
    .unwrap();
    let bytes = node.encode(16_272).unwrap();
    assert_eq!(Node::decode(&bytes), Ok(node));
}

#[test]
fn checksum_and_resealed_directory_damage_are_independently_denied() {
    let mut bytes = leaf().encode(16_272).unwrap();
    bytes[104] ^= 1;
    assert_eq!(Node::decode(&bytes), Err(Denial::IntegrityMismatch));
    reseal(&mut bytes);
    assert_eq!(Node::decode(&bytes), Err(Denial::InvalidDirectory));
}

#[test]
fn wrong_version_oversize_and_unsorted_keys_are_denied() {
    let mut bytes = leaf().encode(16_272).unwrap();
    bytes[8] = 2;
    assert_eq!(Node::decode(&bytes), Err(Denial::UnsupportedVersion));
    assert_eq!(leaf().encode(256), Err(Denial::NodeTooLarge));
    assert_eq!(
        Node::leaf(
            4,
            vec![
                Cell::leaf(b"same".to_vec(), vec![1]),
                Cell::leaf(b"same".to_vec(), vec![2]),
            ],
            None,
            None,
        ),
        Err(Denial::UnsortedKeys)
    );
}

#[test]
fn dedupe_index_real_codec_admits_256_way_nodes_at_64_kib_but_not_16_kib() {
    // DedupeIndex's registered shape is key64/value56. A 64 KiB C.5 page
    // leaves 65_424 payload bytes after framing; the 16 KiB default leaves
    // 16_272. Exercise the actual slotted codec, including cell directories.
    let key = |index: u16| {
        let mut bytes = vec![0_u8; 64];
        bytes[..2].copy_from_slice(&index.to_be_bytes());
        bytes
    };
    let leaf = Node::leaf(
        2,
        (0_u16..256)
            .map(|index| Cell::leaf(key(index), vec![index as u8; 56]))
            .collect(),
        None,
        None,
    )
    .unwrap();
    let leaf_bytes = leaf.encode(65_424).unwrap();
    assert_eq!(leaf.cells().len(), 256);
    assert_eq!(Node::decode(&leaf_bytes), Ok(leaf.clone()));
    assert_eq!(leaf.encode(16_272), Err(Denial::NodeTooLarge));

    // 256 separators means 257 children, satisfying a >=256-way interior.
    let interior = Node::interior(
        2,
        1,
        record(1),
        (0_u16..256)
            .map(|index| Cell::interior(key(index), record(u64::from(index) + 2)))
            .collect(),
        None,
        None,
    )
    .unwrap();
    let interior_bytes = interior.encode(65_424).unwrap();
    assert_eq!(interior.cells().len() + 1, 257);
    assert_eq!(Node::decode(&interior_bytes), Ok(interior.clone()));
    assert_eq!(interior.encode(16_272), Err(Denial::NodeTooLarge));
}
