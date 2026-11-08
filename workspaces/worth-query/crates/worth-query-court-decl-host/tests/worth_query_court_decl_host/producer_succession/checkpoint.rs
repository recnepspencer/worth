//! Observe and corrupt the checkpoint made by the actual Query producer journey.
use super::*;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
const NATIVE_MAGIC: &[u8] = b"WORTH-REL-CHK18\n";
const ARTIFACT_HEADER: usize = 66;
pub fn capture(app: &world::App) -> installation::WorthQueryApplicationCheckpoint {
    app.with_host_advancement(|phase| {
        app.capture_application_checkpoint(
            &phase,
            installation::WorthQueryCheckpointCapturePolicy::SystemAllocation,
        )
        .unwrap()
    })
    .unwrap()
}
#[derive(Deserialize)]
struct Native<'a> {
    #[serde(borrow)]
    members: Vec<(serde::de::IgnoredAny, Blob<'a>)>,
    #[serde(borrow)]
    envelopes: Vec<Blob<'a>>,
}
struct Blob<'a>(&'a [u8]);
impl<'de: 'a, 'a> Deserialize<'de> for Blob<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Blob<'de>;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("borrowed native envelope bytes")
            }
            fn visit_borrowed_bytes<E: serde::de::Error>(
                self,
                bytes: &'de [u8],
            ) -> Result<Self::Value, E> {
                Ok(Blob(bytes))
            }
        }
        d.deserialize_bytes(Visitor)
    }
}
pub fn envelopes(checkpoint: &installation::WorthQueryApplicationCheckpoint) -> Vec<Value> {
    let (_, _, native) = native_wire(checkpoint);
    native
        .members
        .iter()
        .filter(|(_, blob)| blob.0.starts_with(b"WRTHRCCA"))
        .map(|(_, blob)| {
                // Inspection confers no readmission authority. Actual capture
                // supplies the current positioned member; fault fixtures may
                // intentionally leave its integrity commitment invalid.
                worth_relational::facade::durability::RelationalCanonicalCommitArtifact::preflight_frame_version(blob.0).unwrap();
                let length = u64::from_le_bytes(blob.0[58..66].try_into().unwrap());
                assert_eq!(length as usize, blob.0.len() - ARTIFACT_HEADER);
                let super::checkpoint_value::InspectionValue(value) =
                    rmp_serde::from_slice(&blob.0[ARTIFACT_HEADER..]).unwrap();
                value["canonical"].clone()
        })
        .collect()
}
fn native_wire(
    checkpoint: &installation::WorthQueryApplicationCheckpoint,
) -> (usize, usize, Native<'_>) {
    let bytes = checkpoint.bytes();
    let start = find(bytes, NATIVE_MAGIC);
    let length = u64::from_be_bytes(bytes[start - 32..start - 24].try_into().unwrap()) as usize;
    let native: Native<'_> =
        rmp_serde::from_slice(&bytes[start + NATIVE_MAGIC.len()..start + length]).unwrap();
    assert!(
        native.envelopes.is_empty(),
        "current checkpoints carry canonical artifacts as members"
    );
    (start, length, native)
}
fn retirement_member<'a>(native: &'a Native<'_>) -> &'a [u8] {
    native
        .members
        .iter()
        .map(|(_, blob)| blob.0)
        .find(|bytes| {
            bytes.starts_with(b"WRTHRCCA")
                && bytes[ARTIFACT_HEADER..]
                    .windows(b"producer_cascade_retirement".len())
                    .any(|s| s == b"producer_cascade_retirement")
        })
        .expect("actual canonical producer retirement member")
}
fn find(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .position(|s| s == needle)
        .expect("actual native checkpoint field")
}
pub fn tamper_anchor(
    checkpoint: &installation::WorthQueryApplicationCheckpoint,
) -> installation::WorthQueryApplicationCheckpoint {
    let mut bytes = checkpoint.bytes().to_vec();
    let (_, _, native) = native_wire(checkpoint);
    let member = retirement_member(&native);
    let group = find(&bytes, member)
        + ARTIFACT_HEADER
        + find(&member[ARTIFACT_HEADER..], b"producer_cascade_retirement");
    let anchor = group + find(&bytes[group..], b"anchors");
    let slot = anchor + find(&bytes[anchor..], b"local_slot") + b"local_slot".len();
    assert!(
        bytes[slot] < 128,
        "fixture anchor uses a MessagePack positive fixint"
    );
    bytes[slot] ^= 1;
    // Preserve Query framing/checksum. Imported DATA still lacks the issuing
    // capture origin and cannot authorize this changed Native closure.
    let checksum = Sha256::digest(&bytes[40..]);
    bytes[8..40].copy_from_slice(&checksum);
    installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.into_boxed_slice())
}
pub fn assert_shrink_delta(
    app: &world::App,
    before: &installation::WorthQueryApplicationCheckpoint,
    after: &installation::WorthQueryApplicationCheckpoint,
) {
    app.with_host_advancement(|phase| {
        let before = app
            .runtime()
            .inspect_application_checkpoint(&phase, before.bytes())
            .unwrap();
        let after = app
            .runtime()
            .inspect_application_checkpoint(&phase, after.bytes())
            .unwrap();
        let removed = before
            .live_records
            .difference(&after.live_records)
            .collect::<Vec<_>>();
        let entities = removed
            .iter()
            .filter(|r| {
                r.class == worth_relational::facade::durability::RecordAllocationClass::Entity
            })
            .count();
        // Two generated sources, their two realizations, two GeneratedBy and two Realizes edges.
        assert_eq!(
            entities, 4,
            "all retained roots: before={:?}, after={:?}",
            before.live_records, after.live_records
        );
        assert_eq!(removed.len() - entities, 4);
    })
    .unwrap();
}

pub fn omit_leaf_publication(
    checkpoint: &installation::WorthQueryApplicationCheckpoint,
) -> installation::WorthQueryApplicationCheckpoint {
    let (start, native_len, native) = native_wire(checkpoint);
    let blob = retirement_member(&native);
    let group = ARTIFACT_HEADER + find(&blob[ARTIFACT_HEADER..], b"producer_cascade_retirement");
    let array = group + find(&blob[group..], b"publications") + b"publications".len();
    assert_eq!(
        blob[array], 0x92,
        "two separate retired publication origins"
    );
    let first = array + 1;
    let first_len = value_len(&blob[first..]);
    let second = first + first_len;
    let end = second + value_len(&blob[second..]);
    let mut changed = blob.to_vec();
    changed.drain(second..end);
    changed[array] = 0x91;
    replace_member(checkpoint, start, native_len, blob, changed)
}
impl serde::Serialize for Blob<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(self.0)
    }
}
fn value_len(bytes: &[u8]) -> usize {
    let mut decoder = rmp_serde::Deserializer::new(std::io::Cursor::new(bytes));
    serde::de::IgnoredAny::deserialize(&mut decoder).unwrap();
    decoder.position() as usize
}

pub fn omit_retirement_group(
    checkpoint: &installation::WorthQueryApplicationCheckpoint,
) -> installation::WorthQueryApplicationCheckpoint {
    let (start, native_len, native) = native_wire(checkpoint);
    let key = rmp_serde::to_vec("producer_cascade_retirement").unwrap();
    let blob = retirement_member(&native);
    let field = ARTIFACT_HEADER + find(&blob[ARTIFACT_HEADER..], &key);
    let end = field + key.len() + value_len(&blob[field + key.len()..]);
    let map = ARTIFACT_HEADER + find(&blob[ARTIFACT_HEADER..], b"canonical") + b"canonical".len();
    assert_eq!(blob[map], 0xde, "current named envelope uses a map16");
    let count = u16::from_be_bytes(blob[map + 1..map + 3].try_into().unwrap());
    let mut changed = blob.to_vec();
    changed.drain(field..end);
    changed[map + 1..map + 3].copy_from_slice(&(count - 1).to_be_bytes());
    replace_member(checkpoint, start, native_len, blob, changed)
}

fn replace_member(
    checkpoint: &installation::WorthQueryApplicationCheckpoint,
    start: usize,
    native_len: usize,
    original_member: &[u8],
    mut changed: Vec<u8>,
) -> installation::WorthQueryApplicationCheckpoint {
    let payload_length = (changed.len() - ARTIFACT_HEADER) as u64;
    changed[58..66].copy_from_slice(&payload_length.to_le_bytes());
    let original = rmp_serde::to_vec(&Blob(original_member)).unwrap();
    let replacement = rmp_serde::to_vec(&Blob(&changed)).unwrap();
    let bytes = checkpoint.bytes();
    let offset = find(bytes, &original);
    let removed = original.len() - replacement.len();
    let mut frame = bytes.to_vec();
    frame.splice(offset..offset + original.len(), replacement);
    frame[start - 32..start - 24].copy_from_slice(&((native_len - removed) as u64).to_be_bytes());
    let checksum = Sha256::digest(&frame[40..]);
    frame[8..40].copy_from_slice(&checksum);
    installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(frame.into_boxed_slice())
}
