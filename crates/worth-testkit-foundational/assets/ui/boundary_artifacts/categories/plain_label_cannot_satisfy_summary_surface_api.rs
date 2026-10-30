use worth_foundational::{boundary_summary_category_of, FoundationalBoundaryArtifactCategory};

fn main() {
    let label = FoundationalBoundaryArtifactCategory::Summary;
    let _ = boundary_summary_category_of(&label);
}
