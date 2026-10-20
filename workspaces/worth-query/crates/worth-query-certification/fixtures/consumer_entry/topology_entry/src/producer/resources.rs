// The real producer identity framing and demand resource quote.
pub fn planar_source_key(source_identity: &[u8; 32]) -> u64 {
    source_identity
        .chunks_exact(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("eight-byte identity chunk")))
        .fold(0, u64::wrapping_add)
}

pub const fn planar_producer_resources() -> WorthQueryProducerDemandResources {
    WorthQueryProducerDemandResources::new(Some(4_096), 8_192)
}
