#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::read_partitions::*;
use std::collections::BTreeSet;

use worth_store_physical_format::BootstrapCatalog;
use {
    worth_store::physical_runtime::PhysicalSignalAspectBindingObservation,
    worth_store::physical_runtime::PhysicalWorkCausalRecord,
    worth_store::physical_runtime::PhysicalWorkSignalFamily,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::scan_journeys::collect_scan,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

#[test]
fn ordinary_read_and_scan_select_their_exact_store_native_signal_partitions() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &initial,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "physical-work-read-partitions",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
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
    let limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap(),
    );
    let reader = read_serving.records().expect("read protection admission");
    let protected = reader.protected_root();
    assert_eq!(protected.runtime(), read_serving.runtime_identity());
    let catalog = worth_store_physical_format::BootstrapCatalog::decode(
        &std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        protected.root().generation().get(),
        catalog.current_root().generation().get(),
        "the protected read captures the selected on-media root without another root read"
    );
    let (bytes, _) = read_record(reader.open(record, limits).unwrap(), PAYLOAD.len());
    assert_eq!(bytes, PAYLOAD);
    let after_read = read_serving.physical_work_observer().causal().records();
    assert_eq!(
        causal_partitions(&after_read[before_read..], &bindings),
        std::collections::BTreeSet::from([ARTIFACT.to_owned(), FRAME.to_owned()]),
        "ordinary read executes artifact and frame work; root is already captured and protected"
    );

    drop(reader);
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
        std::collections::BTreeSet::from([SCAN.to_owned()]),
        "scan-owned physical work must not launder through ordinary read partitions"
    );
    scan_serving.close();
}
