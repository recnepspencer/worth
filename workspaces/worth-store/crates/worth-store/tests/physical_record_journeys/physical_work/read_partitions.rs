use std::collections::BTreeSet;

use worth_store::physical_runtime::{
    PhysicalSignalAspectBindingObservation, PhysicalWorkCausalRecord, PhysicalWorkSignalFamily,
    RecordAppendBatch, RecordByteLimit, RecordReadLimits,
};

use super::super::{
    durable_publication, read_record, scan_journeys::collect_scan, serving_from_open,
};
use super::{configuration, serving_from_initialization};

const PAYLOAD: &[u8] = b"partition-native canonical record read";
const ROOT: &str = "store.physical.record.root";
const ARTIFACT: &str = "store.physical.record.artifact";
const FRAME: &str = "store.physical.record.frame";
const SCAN: &str = "store.physical.record.scan";

#[test]
fn ordinary_read_and_scan_select_their_exact_store_native_signal_partitions() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = durable_publication::publish_single(
        &initial,
        placement,
        durable_publication::certification_material("physical-work-read-partitions", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let read_serving = serving_from_open(&root);
    let bindings = read_serving.physical_signal_aspect_binding_observations();
    assert_eq!(
        read_binding_partitions(&bindings),
        vec![
            ARTIFACT.to_owned(),
            FRAME.to_owned(),
            ROOT.to_owned(),
            SCAN.to_owned(),
        ],
        "the frozen runtime must install exactly the four bounded read dependencies"
    );

    let before_read = read_serving
        .physical_work_observer()
        .causal()
        .records()
        .len();
    let limits = RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
    let (bytes, _) = read_record(
        read_serving
            .records()
            .expect("read protection admission")
            .open(record, limits)
            .unwrap(),
        PAYLOAD.len(),
    );
    assert_eq!(bytes, PAYLOAD);
    let after_read = read_serving.physical_work_observer().causal().records();
    assert_eq!(
        causal_partitions(&after_read[before_read..], &bindings),
        BTreeSet::from([ARTIFACT.to_owned(), FRAME.to_owned(), ROOT.to_owned()]),
        "ordinary locate and read must bind root, artifact, and frame dependencies"
    );

    read_serving.close();

    let scan_serving = serving_from_open(&root);
    let scan_bindings = scan_serving.physical_signal_aspect_binding_observations();
    let before_scan = scan_serving
        .physical_work_observer()
        .causal()
        .records()
        .len();
    assert_eq!(
        collect_scan(&scan_serving, 1, 4 * 1024),
        vec![(record, PAYLOAD.to_vec())]
    );
    let after_scan = scan_serving.physical_work_observer().causal().records();
    assert_eq!(
        causal_partitions(&after_scan[before_scan..], &scan_bindings),
        BTreeSet::from([SCAN.to_owned()]),
        "scan-owned physical work must not launder through ordinary read partitions"
    );
    scan_serving.close();
}

fn read_binding_partitions(bindings: &[PhysicalSignalAspectBindingObservation]) -> Vec<String> {
    let mut partitions = bindings
        .iter()
        .filter(|binding| {
            binding
                .families()
                .contains(PhysicalWorkSignalFamily::ReadFault)
        })
        .map(|binding| {
            binding
                .partition()
                .map(|partition| partition.partition.0.clone())
                .unwrap_or_else(|| "<unpartitioned>".to_owned())
        })
        .collect::<Vec<_>>();
    partitions.sort();
    partitions
}

fn causal_partitions(
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
                .map(|partition| partition.partition.0.clone())
                .expect("settled read work must identify one installed partitioned binding")
        })
        .collect()
}
