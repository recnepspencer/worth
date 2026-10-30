use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceMilestoneBPerformanceClaimId,
    ResourceMilestoneBPerformanceCloseout, ResourceMilestoneBPerformanceCloseoutRow,
    ResourceMilestoneBPerformanceCloseoutSummary,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn forged_summary() -> ResourceMilestoneBPerformanceCloseoutSummary {
    loop {}
}

fn forged_row() -> ResourceMilestoneBPerformanceCloseoutRow {
    ResourceMilestoneBPerformanceCloseoutRow {
        id: ResourceMilestoneBPerformanceClaimId::RuntimeSummaryReadZeroColdReconstruction,
        evidence_digest: String::new(),
        performance: forged_performance(),
        passed: true,
    }
}

fn main() {
    let _forged = ResourceMilestoneBPerformanceCloseout {
        schema_version: String::new(),
        scenario_matrix_digest: String::new(),
        rows: vec![forged_row()],
        summary: forged_summary(),
        closeout_digest: String::new(),
        passed: true,
    };
}
