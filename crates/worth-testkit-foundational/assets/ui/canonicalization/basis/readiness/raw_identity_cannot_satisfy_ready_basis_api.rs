use worth_foundational::{BoundaryHandle, CanonicalBasisReadyArtifact};

fn requires_ready_basis(_: CanonicalBasisReadyArtifact) {}

fn main() {
    requires_ready_basis(BoundaryHandle::new(7));
}
