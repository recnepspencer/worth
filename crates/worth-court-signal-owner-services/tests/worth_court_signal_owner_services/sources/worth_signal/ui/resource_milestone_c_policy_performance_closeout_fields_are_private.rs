use worth_signal::facade::{
    ResourceBoundaryPerformanceEnvelope, ResourceMilestoneCPolicyPerformanceClaimId,
    ResourceMilestoneCPolicyPerformanceCloseout, ResourceMilestoneCPolicyPerformanceCloseoutRow,
    ResourceMilestoneCPolicyPerformanceCloseoutSummary,
};

fn forged_performance() -> ResourceBoundaryPerformanceEnvelope {
    loop {}
}

fn forged_summary() -> ResourceMilestoneCPolicyPerformanceCloseoutSummary {
    loop {}
}

fn forged_row() -> ResourceMilestoneCPolicyPerformanceCloseoutRow {
    ResourceMilestoneCPolicyPerformanceCloseoutRow {
        id: ResourceMilestoneCPolicyPerformanceClaimId::RegistryFreezeOrderBounded,
        evidence_digest: String::new(),
        policy_provenance_digest: String::new(),
        performance: forged_performance(),
        passed: true,
    }
}

fn main() {
    let _forged = ResourceMilestoneCPolicyPerformanceCloseout {
        schema_version: String::new(),
        scenario_matrix_digest: String::new(),
        rows: vec![forged_row()],
        summary: forged_summary(),
        closeout_digest: String::new(),
        passed: true,
    };
}
