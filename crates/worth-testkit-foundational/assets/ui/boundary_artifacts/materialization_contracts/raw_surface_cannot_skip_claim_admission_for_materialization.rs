use worth_foundational::{
    plan_descriptive_boundary_materialization, FoundationalBoundaryArtifactSurface,
    FoundationalBoundaryMaterializationSeam, FoundationalBoundaryMaterializationSource,
    MaterializedFoundationalProfileSet,
};

fn fake_profile() -> MaterializedFoundationalProfileSet {
    panic!()
}

fn main() {
    let raw_surface = FoundationalBoundaryArtifactSurface::new(vec![1_u8], 0);
    let _ = plan_descriptive_boundary_materialization(
        raw_surface,
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        fake_profile(),
    );
}
