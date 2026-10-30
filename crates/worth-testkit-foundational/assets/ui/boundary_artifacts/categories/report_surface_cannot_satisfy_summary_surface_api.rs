use worth_foundational::{
    boundary_summary_category_of, FoundationalBoundaryReportSurface,
};

fn main() {
    let report = FoundationalBoundaryReportSurface::new(vec!["row"], 1).unwrap();
    let _ = boundary_summary_category_of(&report);
}
