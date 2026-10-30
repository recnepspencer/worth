use worth_signal::facade::{
    ResourceMilestoneCPolicyScenarioMatrix, ResourceMilestoneCPolicyScenarioMatrixSummary,
    ResourceMilestoneCPolicyScenarioRow,
};

fn forged_row() -> ResourceMilestoneCPolicyScenarioRow {
    loop {}
}

fn forged_summary() -> ResourceMilestoneCPolicyScenarioMatrixSummary {
    loop {}
}

fn main() {
    let _forged = ResourceMilestoneCPolicyScenarioMatrix {
        schema_version: String::new(),
        rows: vec![forged_row()],
        summary: forged_summary(),
        matrix_digest: String::new(),
        passed: true,
    };
}
