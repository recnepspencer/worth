use worth_store::physical_runtime::IntactPhysicalAuthorityObservation;
use worth_store_physical_integrity::PhysicalArtifactScope;

fn forge(scope: PhysicalArtifactScope) -> IntactPhysicalAuthorityObservation {
    IntactPhysicalAuthorityObservation::new(scope)
}

fn main() {}
