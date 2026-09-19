use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn content_fingerprint(content: &str) -> [u8; 32] {
    let mut fingerprint = [0; 32];
    for lane in 0_u8..4 {
        let mut hasher = DefaultHasher::new();
        lane.hash(&mut hasher);
        content.hash(&mut hasher);
        let start = usize::from(lane) * 8;
        fingerprint[start..start + 8].copy_from_slice(&hasher.finish().to_le_bytes());
    }
    fingerprint
}
