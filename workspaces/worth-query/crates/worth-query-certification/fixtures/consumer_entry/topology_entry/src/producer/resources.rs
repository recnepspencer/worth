// The real producer identity framing.
pub fn planar_source_key(source_identity: &[u8; 32]) -> u64 {
    source_identity
        .as_chunks::<8>()
        .0
        .iter()
        .map(|chunk| u64::from_le_bytes(*chunk))
        .fold(0, u64::wrapping_add)
}
