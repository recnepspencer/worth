//! Edit sealed input bytes for the hostile recovery cases, preserving untouched fields.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use worth_relational::facade::{
    durability::{RelationalDurableMemberLocator, RelationalNativeCheckpoint},
    history::CommitId,
};

pub(super) const CHECKPOINT_MAGIC: &[u8] = b"WORTH-REL-CHK25\n";
const PAGE_MAGIC: &[u8] = b"WORTH-REL-CHUNK1\n";

pub(super) fn field<'a>(bytes: &'a [u8], name: &str) -> &'a [u8] {
    fields(bytes)
        .into_iter()
        .find(|(key, _)| key == name)
        .unwrap()
        .1
}

pub(super) fn replace_field(bytes: &[u8], name: &str, replacement: &[u8]) -> Vec<u8> {
    let selected = field(bytes, name);
    let start = selected.as_ptr() as usize - bytes.as_ptr() as usize;
    let end = start + selected.len();
    [bytes[..start].as_ref(), replacement, &bytes[end..]].concat()
}

pub(super) fn omit_field(bytes: &[u8], name: &str) -> Vec<u8> {
    let key = rmp_serde::to_vec(name).unwrap();
    let selected = field(bytes, name);
    let value_start = selected.as_ptr() as usize - bytes.as_ptr() as usize;
    let start = value_start - key.len();
    assert_eq!(&bytes[start..value_start], key);
    let mut remaining = [
        bytes[..start].as_ref(),
        &bytes[value_start + selected.len()..],
    ]
    .concat();
    match remaining[0] {
        0x81..=0x8f => remaining[0] -= 1,
        0xde => {
            let n = u16::from_be_bytes(remaining[1..3].try_into().unwrap());
            remaining[1..3].copy_from_slice(&(n - 1).to_be_bytes());
        }
        0xdf => {
            let n = u32::from_be_bytes(remaining[1..5].try_into().unwrap());
            remaining[1..5].copy_from_slice(&(n - 1).to_be_bytes());
        }
        marker => panic!("expected a named map, got {marker:x}"),
    }
    remaining
}

pub(super) fn rows(bytes: &[u8]) -> Vec<&[u8]> {
    let mut cursor = Cursor::new(bytes);
    let count = length(&mut cursor, 0x90, 0xdc, 0xdd);
    (0..count).map(|_| value(&mut cursor)).collect()
}

pub(super) fn array(items: &[&[u8]]) -> Vec<u8> {
    let length = items.len();
    let mut bytes = match length {
        0..=15 => vec![0x90 | u8::try_from(length).unwrap()],
        16..=65535 => [
            vec![0xdc],
            u16::try_from(length).unwrap().to_be_bytes().to_vec(),
        ]
        .concat(),
        _ => [
            vec![0xdd],
            u32::try_from(length).unwrap().to_be_bytes().to_vec(),
        ]
        .concat(),
    };
    for row in items {
        bytes.extend_from_slice(row);
    }
    bytes
}

fn fields(bytes: &[u8]) -> Vec<(String, &[u8])> {
    let mut cursor = Cursor::new(bytes);
    let count = length(&mut cursor, 0x80, 0xde, 0xdf);
    (0..count)
        .map(|_| {
            let name = String::deserialize(&mut rmp_serde::Deserializer::new(&mut cursor)).unwrap();
            (name, value(&mut cursor))
        })
        .collect()
}

fn length(cursor: &mut Cursor<&[u8]>, fixed: u8, short: u8, long: u8) -> usize {
    let mut marker = [0];
    cursor.read_exact(&mut marker).unwrap();
    match marker[0] {
        byte if byte & 0xf0 == fixed => usize::from(byte & 0x0f),
        byte if byte == short => {
            let mut length = [0; 2];
            cursor.read_exact(&mut length).unwrap();
            usize::from(u16::from_be_bytes(length))
        }
        byte if byte == long => {
            let mut length = [0; 4];
            cursor.read_exact(&mut length).unwrap();
            usize::try_from(u32::from_be_bytes(length)).unwrap()
        }
        byte => panic!("unexpected collection marker {byte:#x}"),
    }
}

fn value<'a>(cursor: &mut Cursor<&'a [u8]>) -> &'a [u8] {
    let start = cursor.position() as usize;
    serde::de::IgnoredAny::deserialize(&mut rmp_serde::Deserializer::new(&mut *cursor)).unwrap();
    &cursor.get_ref()[start..cursor.position() as usize]
}

pub(super) struct Bytes<'a>(pub(super) &'a [u8]);
impl Serialize for Bytes<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

#[derive(Deserialize)]
struct CanonicalMemberRow {
    position: u64,
    member: RelationalDurableMemberLocator,
}

pub(super) fn edit_original(
    checkpoint: &RelationalNativeCheckpoint,
    source: &worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource,
    commit_id: CommitId,
    edit: impl FnOnce(&[u8]) -> Vec<u8>,
) -> (
    RelationalNativeCheckpoint,
    std::sync::Arc<
        worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource,
    >,
){
    use worth_relational::facade::durability::RelationalCheckpointMember;
    let checkpoint = checkpoint.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(checkpoint, "manifest");
    let member_bytes = |locator: RelationalDurableMemberLocator| {
        source
            .members
            .get(&locator)
            .expect("external member present in source")
            .bytes()
            .to_vec()
    };
    // CHK24: the edited commit's row lives in one compact commit-index page.
    let index = field(manifest, "commits");
    let mut pages = rows(field(index, "pages"));
    let (page_index, page_frame, row_index, position) = pages
        .iter()
        .enumerate()
        .find_map(|(page_index, page)| {
            let locator: RelationalDurableMemberLocator =
                rmp_serde::from_slice(field(page, "member")).unwrap();
            let frame = member_bytes(locator);
            let payload = &frame[PAGE_MAGIC.len() + 1..];
            let row_index = rows(rows(payload)[1]).iter().position(|row| {
                let receipt: worth_relational::facade::history::RelationalCommitReceipt =
                    rmp_serde::from_slice(rows(row)[1]).unwrap();
                receipt.commit_id == commit_id
            })?;
            let row = rows(rows(payload)[1])[row_index];
            let position: u64 = rmp_serde::from_slice(rows(row)[2]).unwrap();
            Some((page_index, frame, row_index, position))
        })
        .unwrap();
    let mut placements = rows(field(manifest, "canonical_members"));
    let (placement_index, locator) = placements
        .iter()
        .enumerate()
        .find_map(|(index, row)| {
            let row: CanonicalMemberRow = rmp_serde::from_slice(row).unwrap();
            (row.position == position).then_some((index, row.member))
        })
        .unwrap();
    let bytes = member_bytes(locator);

    let (original, digest) = edit_frame(&bytes, edit);
    let locator = RelationalDurableMemberLocator {
        sha256: Sha256::digest(&original).into(),
        length: original.len() as u64,
    };
    let locator_wire = rmp_serde::to_vec_named(&locator).unwrap();

    // Re-sign the page row, its descriptor and any inline head/tail copy.
    let digest = rmp_serde::to_vec(&digest).unwrap();
    let payload = &page_frame[PAGE_MAGIC.len() + 1..];
    let page_rows = rows(rows(payload)[1]);
    let mut row = rows(page_rows[row_index]);
    row[4] = &digest;
    let row = array(&row);
    let mut page_rows = page_rows;
    page_rows[row_index] = &row;
    let page_rows = array(&page_rows);
    let payload = array(&[rows(payload)[0], &page_rows]);
    let frame = [PAGE_MAGIC, &[4], &payload].concat();
    let page_locator = RelationalDurableMemberLocator {
        sha256: Sha256::digest(&frame).into(),
        length: frame.len() as u64,
    };
    let page_locator_wire = rmp_serde::to_vec_named(&page_locator).unwrap();
    let page = replace_field(pages[page_index], "member", &page_locator_wire);
    pages[page_index] = &page;
    let mut index = replace_field(index, "pages", &array(&pages));
    let mut heads = rows(field(&index, "heads"))
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>();
    for head in &mut heads {
        if rmp_serde::from_slice::<u64>(field(head, "position")).unwrap() == position {
            *head = replace_field(head, "payload_digest", &digest);
        }
    }
    index = replace_field(
        &index,
        "heads",
        &array(&heads.iter().map(Vec::as_slice).collect::<Vec<_>>()),
    );
    let tail = field(&index, "tail").to_vec();
    if tail[0] != 0xc0
        && rmp_serde::from_slice::<u64>(field(&tail, "position")).unwrap() == position
    {
        index = replace_field(
            &index,
            "tail",
            &replace_field(&tail, "payload_digest", &digest),
        );
    }
    let changed_placement = replace_field(placements[placement_index], "member", &locator_wire);
    placements[placement_index] = &changed_placement;
    let manifest = replace_field(manifest, "commits", &index);
    let manifest = replace_field(&manifest, "canonical_members", &array(&placements));
    let amended = replace_field(checkpoint, "manifest", &manifest);
    let immutable = |bytes: Vec<u8>| {
        worth_execution::ExecutionImmutableBytes::from_external_bytes(std::sync::Arc::new(
            bytes.into_boxed_slice(),
        ))
    };
    let updated_source = source.with_published_members(&[
        RelationalCheckpointMember::New {
            locator,
            bytes: immutable(original),
        },
        RelationalCheckpointMember::New {
            locator: page_locator,
            bytes: immutable(frame),
        },
    ]);
    (
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &amended].concat()),
        updated_source,
    )
}

pub(super) fn edit_frame(frame: &[u8], edit: impl FnOnce(&[u8]) -> Vec<u8>) -> (Vec<u8>, [u8; 32]) {
    assert_eq!(&frame[..8], b"WRTHRCCA");
    assert_eq!(
        u16::from_le_bytes(frame[8..10].try_into().unwrap()),
        worth_relational::facade::durability::RelationalCanonicalCommitArtifact::FORMAT_VERSION
    );
    let payload = &frame[66..];
    let canonical = edit(field(payload, "canonical"));
    let payload = replace_field(payload, "canonical", &canonical);
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.canonical.v8\0");
    digest.update(&frame[10..26]);
    for key in ["position", "canonical", "original_inputs"] {
        digest.update(Sha256::digest(field(&payload, key)));
    }
    let digest: [u8; 32] = digest.finalize().into();
    let mut amended = frame[..26].to_vec();
    amended.extend_from_slice(&digest);
    amended.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    amended.extend_from_slice(&payload);
    (amended, digest)
}
