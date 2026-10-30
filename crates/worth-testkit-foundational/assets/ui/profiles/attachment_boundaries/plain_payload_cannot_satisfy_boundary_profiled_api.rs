use worth_foundational::BoundaryProfiledArtifact;

fn requires_boundary_profiled(_: BoundaryProfiledArtifact<&'static str>) {}

fn main() {
    requires_boundary_profiled("plain payload");
}
