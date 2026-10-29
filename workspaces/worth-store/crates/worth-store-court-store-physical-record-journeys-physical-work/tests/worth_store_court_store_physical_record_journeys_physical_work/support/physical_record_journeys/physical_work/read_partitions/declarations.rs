pub(crate) use {std::collections::BTreeSet};

pub(crate) use {worth_store::physical_runtime::PhysicalSignalAspectBindingObservation, worth_store::physical_runtime::PhysicalWorkCausalRecord, worth_store::physical_runtime::PhysicalWorkSignalFamily, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};
pub(crate) use {worth_store_physical_format::BootstrapCatalog};

pub(crate) use {worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::read_record, worth_store_testkit_store::physical_record_journeys::scan_journeys::collect_scan, worth_store_testkit_store::physical_record_journeys::serving_from_open};
pub(crate) use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

pub(crate) const PAYLOAD: &[u8] = b"partition-native canonical record read";
pub(crate) const ROOT: &str = "store.physical.record.root";
pub(crate) const ARTIFACT: &str = "store.physical.record.artifact";
pub(crate) const FRAME: &str = "store.physical.record.frame";
pub(crate) const SCAN: &str = "store.physical.record.scan";

pub(crate) fn read_binding_partitions(bindings: &[PhysicalSignalAspectBindingObservation]) -> Vec<String> {
    let mut partitions = bindings
        .iter()
        .filter(|binding| {
            binding
                .families()
                .contains(worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault)
        })
        .map(|binding| {
            binding
                .partition()
                .map(|partition| partition.path().segments()[0].clone())
                .unwrap_or_else(|| "<unpartitioned>".to_owned())
        })
        .collect::<Vec<_>>();
    partitions.sort();
    partitions
}

pub(crate) fn causal_partitions(
    records: &[PhysicalWorkCausalRecord],
    bindings: &[PhysicalSignalAspectBindingObservation],
) -> BTreeSet<String> {
    records
        .iter()
        .map(|record| {
            bindings
                .iter()
                .find(|binding| binding.digest() == record.signal_binding())
                .and_then(|binding| binding.partition())
                .map(|partition| partition.path().segments()[0].clone())
                .expect("settled read work must identify one installed partitioned binding")
        })
        .collect()
}