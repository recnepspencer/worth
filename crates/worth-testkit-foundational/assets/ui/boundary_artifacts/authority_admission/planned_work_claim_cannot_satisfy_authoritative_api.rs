use worth_foundational::{
    claim_planned_work_boundary_surface, FoundationalAuthoritativeBoundaryClaim,
    FoundationalBoundaryArtifactSurface,
};

fn requires_authoritative(
    _: &FoundationalAuthoritativeBoundaryClaim<FoundationalBoundaryArtifactSurface<Vec<u8>>>,
) {
}

fn main() {
    let planned =
        claim_planned_work_boundary_surface(FoundationalBoundaryArtifactSurface::new(vec![1_u8], 0));
    requires_authoritative(&planned);
}
