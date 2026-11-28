use super::*;
#[test]
fn relation_scan_keeps_the_request_charge_failure() {
    for stop in [
        worth_execution::MapKernelStop::Cancelled,
        worth_execution::MapKernelStop::DeadlineElapsed,
        worth_execution::MapKernelStop::WorkCeiling,
    ] {
        let mut charge = |_| Err(stop);
        let mut scan = KindScan::<Relation>::new(Some(8), true, Some(&mut charge), None);
        assert_eq!(
            scan.examine_slot(),
            Err(RequestOr::Request(crate::execution::denial::kernel_stop(
                stop
            )))
        );
        assert_eq!(finish_scan(scan).relation_records_reserved(), 0);
    }
}
