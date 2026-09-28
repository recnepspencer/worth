use worth_foundational::facade::TruthPartitionRole;
use worth_runtime_bridge::facade::{BridgeAuthoritativeSourceProvenance, BridgeProducerMetadata};

fn main() {
    let provenance =
        BridgeAuthoritativeSourceProvenance::from_owner_publication(1, "model", "adapter", "basis");
    let metadata =
        BridgeProducerMetadata::registered_authoritative_source().with_authoritative_source(provenance);
    assert!(metadata.authoritative_source().is_some());

    let partition = TruthPartitionRole::new("structure").expect("valid partition role");
    let partitioned = BridgeAuthoritativeSourceProvenance::from_owner_partition_publication(
        1, "model", "adapter", "basis", partition,
    );
    assert_eq!(partitioned.partition_role().map(TruthPartitionRole::as_str), Some("structure"));
}
