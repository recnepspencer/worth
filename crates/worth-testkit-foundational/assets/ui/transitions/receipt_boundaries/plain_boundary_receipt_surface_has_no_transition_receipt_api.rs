use worth_foundational::FoundationalBoundaryReceiptSurface;

fn main() {
    let receipt = FoundationalBoundaryReceiptSurface::new("completed", 1).unwrap();
    let _ = receipt.transition_provenance_rows();
}
