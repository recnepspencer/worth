use worth_signal::facade::ResourceTimeoutHeartbeatExtensionReport;

fn touch(report: ResourceTimeoutHeartbeatExtensionReport) {
    let _ = report.extended_heartbeat;
    let _ = report.denied_extension;
    let _ = report.performance;
}

fn main() {}
