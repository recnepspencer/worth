//! Observe and corrupt the checkpoint made by the actual Query producer journey.
use super::*;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
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
    let bytes = checkpoint.bytes();
    let start = find(bytes, b"WORTH-REL-CHK20.1\n");
    let len = u64::from_be_bytes(bytes[start - 16..start - 8].try_into().unwrap()) as usize;
    let native: Native<'_> =
        rmp_serde::from_slice(&bytes[start + b"WORTH-REL-CHK20.1\n".len()..start + len]).unwrap();
    native
        .envelopes
        .iter()
        .map(|blob| {
            let super::checkpoint_value::InspectionValue(value) =
                rmp_serde::from_slice(blob.0).unwrap();
            value["Native"]["canonical"].clone()
        })
        .collect()
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
    let group = find(&bytes, b"producer_cascade_retirement");
    let anchor = group + find(&bytes[group..], b"anchors");
    let slot = anchor + find(&bytes[anchor..], b"local_slot") + b"local_slot".len();
    assert!(
        bytes[slot] < 128,
        "fixture anchor uses a MessagePack positive fixint"
    );
    bytes[slot] ^= 1;
    // Preserve Query framing/checksum so Native judges the changed closure.
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
            .filter(|record| !before.current_publication_records.contains(record))
            .collect::<Vec<_>>();
        let entities = removed
            .iter()
            .filter(|r| {
                r.class == worth_relational::facade::durability::RecordAllocationClass::Entity
            })
            .count();
        // Two generated sources, their two realizations, two GeneratedBy and two Realizes edges.
        assert_eq!(entities, 4);
        assert_eq!(removed.len() - entities, 4);
    })
    .unwrap();
}

pub fn omit_leaf_publication(
    checkpoint: &installation::WorthQueryApplicationCheckpoint,
) -> installation::WorthQueryApplicationCheckpoint {
    let bytes = checkpoint.bytes();
    let start = find(bytes, b"WORTH-REL-CHK20.1\n");
    let native_len = u64::from_be_bytes(bytes[start - 16..start - 8].try_into().unwrap()) as usize;
    let native: Native<'_> =
        rmp_serde::from_slice(&bytes[start + b"WORTH-REL-CHK20.1\n".len()..start + native_len])
            .unwrap();
    let blob = native
        .envelopes
        .iter()
        .find(|b| {
            b.0.windows(b"producer_cascade_retirement".len())
                .any(|s| s == b"producer_cascade_retirement")
        })
        .unwrap();
    let group = find(blob.0, b"producer_cascade_retirement");
    let array = group + find(&blob.0[group..], b"publications") + b"publications".len();
    assert_eq!(
        blob.0[array], 0x92,
        "two separate retired publication origins"
    );
    let first = array + 1;
    let first_len = value_len(&blob.0[first..]);
    let second = first + first_len;
    let end = second + value_len(&blob.0[second..]);
    let mut changed = blob.0.to_vec();
    changed.drain(second..end);
    changed[array] = 0x91;
    let original = rmp_serde::to_vec(blob).unwrap();
    let replacement = rmp_serde::to_vec(&Blob(&changed)).unwrap();
    let offset = find(bytes, &original);
    let removed = original.len() - replacement.len();
    let mut frame = bytes.to_vec();
    frame.splice(offset..offset + original.len(), replacement);
    frame[start - 16..start - 8].copy_from_slice(&((native_len - removed) as u64).to_be_bytes());
    let checksum = Sha256::digest(&frame[40..]);
    frame[8..40].copy_from_slice(&checksum);
    installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(frame.into_boxed_slice())
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
    let bytes = checkpoint.bytes();
    let start = find(bytes, b"WORTH-REL-CHK20.1\n");
    let native_len = u64::from_be_bytes(bytes[start - 16..start - 8].try_into().unwrap()) as usize;
    let native: Native<'_> =
        rmp_serde::from_slice(&bytes[start + b"WORTH-REL-CHK20.1\n".len()..start + native_len])
            .unwrap();
    let key = rmp_serde::to_vec("producer_cascade_retirement").unwrap();
    let blob = native
        .envelopes
        .iter()
        .find(|b| b.0.windows(key.len()).any(|s| s == key))
        .unwrap();
    let field = find(blob.0, &key);
    let end = field + key.len() + value_len(&blob.0[field + key.len()..]);
    let map = find(blob.0, b"canonical") + b"canonical".len();
    assert_eq!(blob.0[map], 0xde, "current named envelope uses a map16");
    let count = u16::from_be_bytes(blob.0[map + 1..map + 3].try_into().unwrap());
    let mut changed = blob.0.to_vec();
    changed.drain(field..end);
    changed[map + 1..map + 3].copy_from_slice(&(count - 1).to_be_bytes());
    let original = rmp_serde::to_vec(blob).unwrap();
    let replacement = rmp_serde::to_vec(&Blob(&changed)).unwrap();
    let offset = find(bytes, &original);
    let removed = original.len() - replacement.len();
    let mut frame = bytes.to_vec();
    frame.splice(offset..offset + original.len(), replacement);
    frame[start - 16..start - 8].copy_from_slice(&((native_len - removed) as u64).to_be_bytes());
    let checksum = Sha256::digest(&frame[40..]);
    frame[8..40].copy_from_slice(&checksum);
    installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(frame.into_boxed_slice())
}
