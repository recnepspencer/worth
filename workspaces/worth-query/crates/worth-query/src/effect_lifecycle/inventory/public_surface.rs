use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::kinds::{
    EffectPublicSurfaceAvailability, EffectPublicSurfaceKind, EffectReceiptArtifactKind,
};
use super::EFFECT_LIFECYCLE_IDENTITY_SCOPE;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecyclePublicSurfaceRow {
    surface_kind: EffectPublicSurfaceKind,
    entrypoint: Option<&'static str>,
    primary_artifact_kind: Option<EffectReceiptArtifactKind>,
    availability: EffectPublicSurfaceAvailability,
    lower_runtime_visibility_hidden: bool,
    row_identity: WorthQueryEvidenceIdentity,
}

impl EffectLifecyclePublicSurfaceRow {
    pub(in crate::effect_lifecycle) fn new(
        surface_kind: EffectPublicSurfaceKind,
        entrypoint: Option<&'static str>,
        primary_artifact_kind: Option<EffectReceiptArtifactKind>,
        availability: EffectPublicSurfaceAvailability,
        lower_runtime_visibility_hidden: bool,
    ) -> Result<Self, IdentityDerivationDenial> {
        let row_identity = WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_lifecycle_public_surface_row_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("surface_kind"),
                surface_kind.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("entrypoint"),
                entrypoint.unwrap_or("none"),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("artifact"),
                primary_artifact_kind
                    .map(|kind| kind.as_str())
                    .unwrap_or("none"),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("availability"),
                availability.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("hidden"),
                lower_runtime_visibility_hidden.to_string().as_str(),
            )
            .seal()?;
        Ok(Self {
            surface_kind,
            entrypoint,
            primary_artifact_kind,
            availability,
            lower_runtime_visibility_hidden,
            row_identity,
        })
    }

    pub fn surface_kind(&self) -> EffectPublicSurfaceKind {
        self.surface_kind
    }
    pub fn entrypoint(&self) -> Option<&'static str> {
        self.entrypoint
    }
    pub fn primary_artifact_kind(&self) -> Option<EffectReceiptArtifactKind> {
        self.primary_artifact_kind
    }
    pub fn availability(&self) -> EffectPublicSurfaceAvailability {
        self.availability
    }
    pub fn lower_runtime_visibility_hidden(&self) -> bool {
        self.lower_runtime_visibility_hidden
    }
    pub fn row_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.row_identity
    }
    pub fn row_for_reporting(&self) -> &str {
        self.row_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectLifecyclePublicSurfaceInventory {
    pub(super) rows: Vec<EffectLifecyclePublicSurfaceRow>,
    pub(super) inventory_identity: WorthQueryEvidenceIdentity,
}

impl EffectLifecyclePublicSurfaceInventory {
    pub fn rows(&self) -> &[EffectLifecyclePublicSurfaceRow] {
        &self.rows
    }
    pub fn inventory_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inventory_identity
    }
    pub fn inventory_for_reporting(&self) -> &str {
        self.inventory_identity.as_str()
    }
}
