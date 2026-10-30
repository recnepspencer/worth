use worth_foundational::{BoundaryArtifactId, BoundaryHandle};

fn needs_handle(_handle: BoundaryHandle) {}

fn main() {
    let artifact_id = BoundaryArtifactId::new(1);
    needs_handle(artifact_id);
}
