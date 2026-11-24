use crate::replay::data::digest_derived_index_summary;
use crate::replay::data::digest_derived_index_surface;
use crate::replay::data::ReplayObservableSurface;
use crate::replay::data::ReplaySurfaceAuthorityKind;
use crate::replay::data::ReplaySurfaceComparisonBasis;
use crate::replay::data::VerifiedReplaySurfaceDigest;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexArtifacts;

pub(super) fn derived_index_surface_is_promised(
    original: &CanonicalCommitEnvelope,
    recovered: Option<&CanonicalCommitEnvelope>,
) -> bool {
    !original.derived_index_artifacts().is_empty()
        || recovered.is_some_and(|envelope| !envelope.derived_index_artifacts().is_empty())
}

pub fn surface_basis_for_derived_index_artifacts(
    artifacts: &DerivedIndexArtifacts,
) -> ReplaySurfaceComparisonBasis {
    ReplaySurfaceComparisonBasis::new(
        ReplaySurfaceAuthorityKind::DerivedIndexes,
        Some(VerifiedReplaySurfaceDigest::from_digest(
            ReplaySurfaceAuthorityKind::DerivedIndexes,
            digest_derived_index_surface(artifacts),
        )),
        Some(digest_derived_index_summary(artifacts)),
    )
}

pub const DERIVED_INDEX_SURFACE: ReplayObservableSurface = ReplayObservableSurface::DerivedIndexes;
