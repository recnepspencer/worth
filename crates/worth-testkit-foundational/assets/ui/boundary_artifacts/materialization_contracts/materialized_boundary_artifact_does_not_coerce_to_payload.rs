use worth_foundational::{
    FoundationalBoundaryArtifactSurface, FoundationalMaterializedBoundaryArtifact,
};

fn requires_payload(_: &[u8]) {}

fn fake_materialized(
) -> FoundationalMaterializedBoundaryArtifact<FoundationalBoundaryArtifactSurface<Vec<u8>>> {
    panic!()
}

fn main() {
    let materialized = fake_materialized();
    requires_payload(&materialized);
}
