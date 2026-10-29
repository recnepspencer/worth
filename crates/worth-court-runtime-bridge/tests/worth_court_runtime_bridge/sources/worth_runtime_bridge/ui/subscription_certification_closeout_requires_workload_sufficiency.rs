use worth_runtime_bridge::facade::{
    BridgeSubscriptionReferenceWorkloadReport,
    BridgeSubscriptionReferenceWorkloadSufficiency,
    BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest,
};

fn requires_sufficiency(_: BridgeSubscriptionReferenceWorkloadSufficiency) {}

fn main() {
    let report = loop {};
    let _: BridgeSubscriptionReferenceWorkloadReport = report;
    requires_sufficiency(report);
    let _ = std::mem::size_of::<BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest>();
}
