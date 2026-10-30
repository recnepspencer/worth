use worth_foundational::{
    BoundaryProfiledArtifact, ProofBearingProfiledArtifact,
};

fn requires_proof_bearing_profiled(_: ProofBearingProfiledArtifact<&'static str>) {}
fn boundary_profiled() -> BoundaryProfiledArtifact<&'static str> {
    panic!("type-check only")
}

fn main() {
    requires_proof_bearing_profiled(boundary_profiled());
}
