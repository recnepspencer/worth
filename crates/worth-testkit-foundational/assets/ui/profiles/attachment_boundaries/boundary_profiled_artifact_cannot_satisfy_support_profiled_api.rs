use worth_foundational::{
    BoundaryProfiledArtifact, SupportProfiledArtifact,
};

fn requires_support_profiled(_: SupportProfiledArtifact<&'static str>) {}
fn boundary_profiled() -> BoundaryProfiledArtifact<&'static str> {
    panic!("type-check only")
}

fn main() {
    requires_support_profiled(boundary_profiled());
}
