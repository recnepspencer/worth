#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::read_partition_failures::*;
use std::collections::BTreeSet;

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
