use worth_runtime_bridge::facade::{BridgeAuthoritativeSourceProvenance, BridgeProducerMetadata};

fn main() {
    let provenance =
        BridgeAuthoritativeSourceProvenance::from_owner_publication(1, "model", "adapter", "basis");
    let metadata =
        BridgeProducerMetadata::registered_authoritative_source().with_authoritative_source(provenance);
    assert!(metadata.authoritative_source().is_some());
}
