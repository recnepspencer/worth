use worth_foundational::{
    admit_authoritative_current_boundary_surface, foundational_boundary_authority_admission,
    FoundationalBoundarySummarySurface,
};

fn main() {
    let summary = FoundationalBoundarySummarySurface::new("overview", 1).unwrap();
    let _ = admit_authoritative_current_boundary_surface(
        summary,
        foundational_boundary_authority_admission(),
    );
}
