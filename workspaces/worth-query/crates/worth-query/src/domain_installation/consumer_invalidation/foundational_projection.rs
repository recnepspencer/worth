use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::facade::{
    AspectFieldLocator, AspectLocator, AspectMask, AspectMaskLocator, AspectValueLocator,
    BoundaryArtifactField, BoundaryArtifactLocator, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceProvenanceArtifact, LocatorAuthority, ProjectionMask,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryFoundationalInvalidationScope {
    locator: AspectValueLocator,
    mask: AspectMaskLocator<ProjectionMask>,
}

impl WorthQueryFoundationalInvalidationScope {
    pub const fn locator(&self) -> &AspectValueLocator {
        &self.locator
    }

    pub const fn mask(&self) -> &AspectMaskLocator<ProjectionMask> {
        &self.mask
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryFoundationalInvalidationProjection {
    scopes: Vec<WorthQueryFoundationalInvalidationScope>,
    locality: super::WorthQueryConsumerInvalidationLocality,
    semantic_boundary: BoundaryArtifactLocator,
    provenance: FoundationalBoundaryEvidenceProvenanceArtifact,
}

impl WorthQueryFoundationalInvalidationProjection {
    pub fn scopes(&self) -> &[WorthQueryFoundationalInvalidationScope] {
        &self.scopes
    }

    pub const fn locality(&self) -> super::WorthQueryConsumerInvalidationLocality {
        self.locality
    }

    pub const fn semantic_boundary(&self) -> &BoundaryArtifactLocator {
        &self.semantic_boundary
    }

    pub const fn provenance(&self) -> &FoundationalBoundaryEvidenceProvenanceArtifact {
        &self.provenance
    }
}

impl super::WorthQueryConsumerInvalidationDelta {
    pub fn foundational_projection(
        &self,
    ) -> Result<WorthQueryFoundationalInvalidationProjection, IdentityDerivationDenial> {
        foundational_projection(
            self,
            FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        )
    }
}

pub(super) fn foundational_projection(
    delta: &super::WorthQueryConsumerInvalidationDelta,
    freshness: FoundationalBoundaryEvidenceFreshnessPosture,
) -> Result<WorthQueryFoundationalInvalidationProjection, IdentityDerivationDenial> {
    let semantic_boundary = BoundaryArtifactLocator::new(
        super::foundational_identity::descriptive_boundary_id(delta)?,
        BoundaryArtifactField::Payload,
    );
    let scopes =
        if delta.locality() == super::WorthQueryConsumerInvalidationLocality::DeclaredNativeKeys {
            delta
                .affected_native_keys()
                .iter()
                .map(foundational_scope)
                .collect()
        } else {
            Vec::new()
        };
    Ok(WorthQueryFoundationalInvalidationProjection {
        scopes,
        locality: delta.locality(),
        semantic_boundary,
        provenance: super::foundational_identity::provenance(semantic_boundary, freshness),
    })
}

pub(super) fn foundational_scope(
    key: &crate::domain_installation::WorthQueryNativeAccessKey,
) -> WorthQueryFoundationalInvalidationScope {
    let aspect = AspectLocator::new(LocatorAuthority::Projected, key.contract_key().clone());
    let path = key
        .field_path()
        .canonical_field_path()
        .cloned()
        .or_else(|| {
            key.field_path()
                .native_field_key()
                .cloned()
                .map(worth_foundational::facade::CanonicalFieldPath::single)
        });
    let (locator, mask) = match path {
        Some(path) => {
            let mask = AspectMask::new([path.clone()]);
            (
                AspectValueLocator::struct_field(AspectFieldLocator::from_aspect(aspect, path)),
                AspectMaskLocator::projection(
                    LocatorAuthority::Projected,
                    key.contract_key().clone(),
                    &mask,
                ),
            )
        }
        None => {
            let mask = AspectMask::whole_aspect();
            (
                AspectValueLocator::whole_aspect(aspect),
                AspectMaskLocator::projection(
                    LocatorAuthority::Projected,
                    key.contract_key().clone(),
                    &mask,
                ),
            )
        }
    };
    WorthQueryFoundationalInvalidationScope { locator, mask }
}
