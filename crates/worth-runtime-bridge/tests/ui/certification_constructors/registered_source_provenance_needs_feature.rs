use worth_runtime_bridge::facade::{BridgeAuthoritativeSourceProvenance, BridgeProducerMetadata};

fn main() {
    let provenance =
        BridgeAuthoritativeSourceProvenance::from_owner_publication(1, "model", "adapter", "basis");
    let _partitioned = BridgeAuthoritativeSourceProvenance::from_owner_partition_publication(
        1,
        "model",
        "adapter",
        "basis",
        sealed_authority_placeholder(),
    );
    let _registered =
        BridgeProducerMetadata::registered_authoritative_source().with_authoritative_source(provenance);
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
