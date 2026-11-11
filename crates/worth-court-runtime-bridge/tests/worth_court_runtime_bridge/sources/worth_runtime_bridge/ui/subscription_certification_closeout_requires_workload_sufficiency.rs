use worth_runtime_bridge::facade::{
    BridgeSubscriptionReferenceWorkloadReport, BridgeSubscriptionReferenceWorkloadSufficiency,
    BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest,
};

fn requires_sufficiency(_: BridgeSubscriptionReferenceWorkloadSufficiency) {}

fn main() {
    let report = supplied();
    let _: BridgeSubscriptionReferenceWorkloadReport = report;
    requires_sufficiency(report);
    let _ = std::mem::size_of::<BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest>();
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
