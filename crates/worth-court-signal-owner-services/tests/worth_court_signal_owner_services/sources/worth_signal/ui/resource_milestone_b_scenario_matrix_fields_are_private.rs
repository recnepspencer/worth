use worth_signal::facade::{
    ResourceMilestoneBScenarioMatrix, ResourceMilestoneBScenarioMatrixSummary,
    ResourceMilestoneBScenarioRow,
};

fn forged_row() -> ResourceMilestoneBScenarioRow {
    loop {}
}

fn forged_summary() -> ResourceMilestoneBScenarioMatrixSummary {
    loop {}
}

fn main() {
    let _forged = ResourceMilestoneBScenarioMatrix {
        schema_version: String::new(),
        bundle_digest: String::new(),
        rows: vec![forged_row()],
        summary: forged_summary(),
        matrix_digest: String::new(),
        passed: true,
    };
}
