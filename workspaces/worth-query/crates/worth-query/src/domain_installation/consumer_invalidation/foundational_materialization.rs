use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::{
    claim_derived_projection_boundary_surface, materialize_descriptive_boundary_surface,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryMaterializationDenial, FoundationalBoundaryMaterializationSeam,
    FoundationalBoundaryMaterializationSource, FoundationalMaterializedBoundaryArtifact,
    MaterializedFoundationalProfileSet,
};

pub type WorthQueryFoundationalInvalidationBoundaryArtifact =
    FoundationalMaterializedBoundaryArtifact<
        FoundationalBoundaryArtifactSurface<super::WorthQueryFoundationalInvalidationProjection>,
    >;

pub enum WorthQueryFoundationalInvalidationMaterializationStop {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    ForeignOrStaleLease,
    Foundational(FoundationalBoundaryMaterializationDenial),
}

impl super::WorthQueryAdmittedConsumerInvalidation<'_> {
    pub fn foundational_projection(
        &self,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> Result<Option<super::WorthQueryFoundationalInvalidationProjection>, IdentityDerivationDenial>
    {
        self.remains_current(workspace)
            .then(|| {
                super::foundational_projection::foundational_projection(
                    self.delta(),
                    FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
                )
            })
            .transpose()
    }

    pub fn materialize_foundational_projection(
        &self,
        workspace: &crate::runtime::WorthQueryWorkspace,
        profile: MaterializedFoundationalProfileSet,
    ) -> Result<
        WorthQueryFoundationalInvalidationBoundaryArtifact,
        WorthQueryFoundationalInvalidationMaterializationStop,
    > {
        let projection = self
            .foundational_projection(workspace)
            .map_err(WorthQueryFoundationalInvalidationMaterializationStop::EvidenceIdentityDerivationDenied)?
            .ok_or(WorthQueryFoundationalInvalidationMaterializationStop::ForeignOrStaleLease)?;
        let surface = FoundationalBoundaryArtifactSurface::new(projection, 1);
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(surface),
            FoundationalBoundaryMaterializationSource::NativeAuthority,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .map_err(WorthQueryFoundationalInvalidationMaterializationStop::Foundational)
    }
}
