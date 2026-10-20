use super::*;
#[test]
fn relation_scan_keeps_the_request_charge_failure() {
    for stop in [
        worth_execution::MapKernelStop::Cancelled,
        worth_execution::MapKernelStop::DeadlineElapsed,
        worth_execution::MapKernelStop::WorkCeiling,
    ] {
        let mut charge = |_| Err(stop);
        let mut scan = RelationKindScan::funded(8, true, &mut charge, None);
        assert_eq!(
            scan.examine_slot(),
            Err(NativeScanStop::Request(
                crate::execution::denial::kernel_stop(stop)
            ))
        );
        assert_eq!(scan.finish().relation_records_reserved(), 0);
    }
}
