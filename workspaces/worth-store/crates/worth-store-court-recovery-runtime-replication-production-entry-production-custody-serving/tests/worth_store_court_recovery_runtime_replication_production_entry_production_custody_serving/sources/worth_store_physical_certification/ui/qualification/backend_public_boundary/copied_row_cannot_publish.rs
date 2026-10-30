use worth_store_physical_certification::{BackendQualificationRow, QualificationMatrixPublisher};

fn main() {
    let row: BackendQualificationRow = todo!();
    let _ = QualificationMatrixPublisher::from_executed_store_evidence().with_row(row);
}
