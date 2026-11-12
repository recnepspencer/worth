//! Ordinary fixture input mutation preserves every unrelated numeric token.
pub(in super::super) fn omit_nil_record_field<T: serde::Serialize>(
    record: &T,
    name: &str,
    map_at: usize,
) -> Vec<u8> {
    let mut bytes = rmp_serde::to_vec_named(record).unwrap();
    let mut key = rmp_serde::to_vec(name).unwrap();
    key.push(0xc0);
    let at = unique(&bytes, &key);
    assert!(at > map_at);
    match bytes[map_at] {
        0x81..=0x8f => bytes[map_at] -= 1,
        0xde => {
            let count = u16::from_be_bytes(bytes[map_at + 1..map_at + 3].try_into().unwrap());
            bytes[map_at + 1..map_at + 3].copy_from_slice(&(count - 1).to_be_bytes());
        }
        _ => panic!("actual selected record must use fixmap or map16"),
    }
    bytes.drain(at..at + key.len());
    bytes
}
pub(in super::super) fn replace_record<T: serde::Serialize>(
    payload: &[u8],
    record: &T,
    changed: &[u8],
) -> Vec<u8> {
    let original = rmp_serde::to_vec_named(record).unwrap();
    let at = unique(payload, &original);
    let mut bytes = payload[..at].to_vec();
    bytes.extend_from_slice(changed);
    bytes.extend_from_slice(&payload[at + original.len()..]);
    bytes
}
fn unique(bytes: &[u8], needle: &[u8]) -> usize {
    let matches: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == needle).then_some(at))
        .collect();
    assert_eq!(matches.len(), 1, "actual selected record/field is unique");
    matches[0]
}
