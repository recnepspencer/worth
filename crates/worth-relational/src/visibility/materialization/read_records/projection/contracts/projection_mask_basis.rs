use worth_foundational::facade::{
    prepare_aspect_mask_for_canonical_basis, AspectKey, AspectMask, CanonicalBasisReadyArtifact,
    CanonicalizationRuleVersion, ProjectionMask,
};
use worth_proof::TransitionOutcome;

pub(super) type ProjectionMaskCanonicalBasis = CanonicalBasisReadyArtifact;

pub(super) fn prepare_projection_mask_for_canonical_basis(
    aspect_key: &AspectKey,
    mask: &AspectMask<ProjectionMask>,
) -> Option<ProjectionMaskCanonicalBasis> {
    let version =
        CanonicalizationRuleVersion::new("worth.relational.visibility.projection_scope.v1")
            .expect(/* Proof: This constructor receives a fixed nonempty literal in the required grammar and version/domain; it has no runtime-dependent input. */ "projection scope canonicalization version is static and non-empty");
    match prepare_aspect_mask_for_canonical_basis(version, aspect_key.clone(), mask.clone()) {
        TransitionOutcome::Success(ready) => Some(ready),
        TransitionOutcome::Denied(_)
        | TransitionOutcome::Deferred(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => None,
    }
}
