use worth_signal::facade::adapters::{FrontierRouteEvidenceReason, FrontierRouteEvidenceReceipt};

fn main() {
    let _ = FrontierRouteEvidenceReceipt {
        reason: FrontierRouteEvidenceReason::SerialExecutor,
    };
}
