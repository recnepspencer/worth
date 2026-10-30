use worth_foundational::{
    claim_receipt_evidence_boundary_surface, FoundationalBoundaryReportSurface,
};

fn main() {
    let report = FoundationalBoundaryReportSurface::new(vec!["row"], 1).unwrap();
    let _ = claim_receipt_evidence_boundary_surface(report);
}
