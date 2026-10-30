#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::read_partition_failures::*;
use {
    std::collections::BTreeMap, std::collections::BTreeSet, std::path::Path, std::time::Duration,
    std::time::Instant,
};

use {
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::PhysicalSignalAspectBindingObservation,
    worth_store::physical_runtime::PhysicalWorkCausalRecord,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaOperationRole,
};

use worth_store_testkit_store::durable_publication;
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

#[test]
fn every_transient_read_route_preserves_truth_then_allows_same_runtime_retry() {
    let mut targets = calibrate(
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Ordinary,
    );
    targets.extend(calibrate(
        crate::physical_record_journeys::physical_work::read_partition_failures::Workload::Scan,
    ));
    assert_eq!(
        targets
            .iter()
            .map(|target| target.route)
            .collect::<BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            crate::physical_record_journeys::physical_work::read_partition_failures::Route::Frame,
            crate::physical_record_journeys::physical_work::read_partition_failures::Route::Scan
        ]),
        "calibration must cover the positioned-read routes actually executed after root capture"
    );
    for target in targets {
        exercise(target);
    }
}
