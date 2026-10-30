use worth_foundational::{
    claim_support_only_boundary_surface, FoundationalAuthoritativeBoundaryClaim,
    FoundationalBoundaryArtifactSurface,
};

fn requires_authoritative(
    _: &FoundationalAuthoritativeBoundaryClaim<FoundationalBoundaryArtifactSurface<Vec<u8>>>,
) {
}

fn main() {
    let support =
        claim_support_only_boundary_surface(FoundationalBoundaryArtifactSurface::new(vec![1_u8], 0));
    requires_authoritative(&support);
}
