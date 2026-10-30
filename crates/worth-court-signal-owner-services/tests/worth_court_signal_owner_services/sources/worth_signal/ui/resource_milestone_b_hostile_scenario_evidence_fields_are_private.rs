use worth_signal::facade::{
    ResourceMilestoneBHostileScenarioEvidence, ResourceMilestoneBHostileScenarioEvidenceRow,
};

fn forged_row() -> ResourceMilestoneBHostileScenarioEvidenceRow {
    loop {}
}

fn main() {
    let _forged = ResourceMilestoneBHostileScenarioEvidence {
        schema_version: String::new(),
        rows: vec![forged_row()],
        evidence_digest: String::new(),
    };
}
