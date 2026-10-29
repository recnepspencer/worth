use worth_foundational::{
    claim_derived_projection_boundary_surface, FoundationalBoundaryReceiptSurface,
};

fn main() {
    let receipt = FoundationalBoundaryReceiptSurface::new("completed", 1).unwrap();
    let _ = claim_derived_projection_boundary_surface(receipt);
}
