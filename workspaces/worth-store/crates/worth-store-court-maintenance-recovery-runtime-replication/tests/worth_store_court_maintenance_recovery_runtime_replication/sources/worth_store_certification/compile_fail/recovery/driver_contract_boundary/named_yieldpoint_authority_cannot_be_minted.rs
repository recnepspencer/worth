use worth_store_physical_certification::{PhysicalBoundarySeam, PhysicalBoundaryYieldpoint};

fn main() {
    let _forged = PhysicalBoundaryYieldpoint::named_production_boundary(
        "root-publication-before-observe",
        PhysicalBoundarySeam::MemoryPressure,
    );
}
