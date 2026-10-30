use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectFieldLocator, AspectKey, AspectLocator};

use crate::identity::{BridgeIdentity, TruthDeltaSurfaceIdentityTag};
use crate::input::envelope::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeCommittedPatchTarget,
};
use crate::mapping::{FrozenAspectMappingRegistry, TruthDeltaSurfaceKind, TruthPatchTargetView};
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;

pub(crate) type TruthDeltaSurfaceIdentity = BridgeIdentity<TruthDeltaSurfaceIdentityTag>;

enum TruthDeltaSurfaceTargetMaskIdentityTag {}

type TruthDeltaSurfaceTargetMaskIdentity = BridgeIdentity<TruthDeltaSurfaceTargetMaskIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthDeltaSurface {
    surface_identity: TruthDeltaSurfaceIdentity,
    entity_identity: Arc<str>,
    relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    target: BridgeCommittedPatchTarget,
    native_target_basis: Arc<str>,
}

impl PartialOrd for TruthDeltaSurface {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TruthDeltaSurface {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.surface_identity.cmp(&other.surface_identity)
    }
}

impl TruthDeltaSurface {
    fn from_native_target_parts(
        entity_identity: impl Into<Arc<str>>,
        relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
        target: BridgeCommittedPatchTarget,
        native_target_basis: Arc<str>,
    ) -> Self {
        let entity_identity = entity_identity.into();
        let target_mask_identity = truth_delta_surface_target_mask_identity(&target);
        let surface_identity = truth_delta_surface_identity(
            entity_identity.as_ref(),
            target.surface_kind(),
            &target_mask_identity,
        );

        Self {
            surface_identity,
            entity_identity,
            relational_record_identity,
            target,
            native_target_basis,
        }
    }

    pub(crate) fn entity_identity(&self) -> &str {
        self.entity_identity.as_ref()
    }

    pub(crate) fn relational_record_identity_parts(
        &self,
    ) -> Option<RelationalBridgeRecordIdentityParts> {
        self.relational_record_identity
    }

    pub(crate) fn aspect_key(&self) -> &AspectKey {
        self.target.aspect_key()
    }

    pub(crate) fn aspect_locator(&self) -> &AspectLocator {
        self.target.aspect_locator()
    }

    pub(crate) fn field_locator(&self) -> Option<&AspectFieldLocator> {
        self.target.field_locator()
    }

    pub(crate) fn projection_mask(
        &self,
    ) -> &worth_foundational::facade::AspectMask<worth_foundational::facade::ProjectionMask> {
        self.target.projection_mask()
    }

    pub(crate) fn target(&self) -> &BridgeCommittedPatchTarget {
        &self.target
    }

    pub(crate) fn surface_kind(&self) -> TruthDeltaSurfaceKind {
        self.target.surface_kind()
    }

    pub(crate) fn surface_identity(&self) -> &TruthDeltaSurfaceIdentity {
        &self.surface_identity
    }

    pub(crate) fn native_target_basis(&self) -> &str {
        self.native_target_basis.as_ref()
    }
}

fn truth_delta_surface_identity(
    entity_identity: &str,
    surface_kind: TruthDeltaSurfaceKind,
    target_mask_identity: &TruthDeltaSurfaceTargetMaskIdentity,
) -> TruthDeltaSurfaceIdentity {
    let basis = format!(
        "truth-delta-surface|entity={entity_identity}|kind={}|target-mask-proof={}",
        canonical_truth_surface_kind_label(surface_kind),
        target_mask_identity.as_str(),
    );
    let digest = Sha256::digest(basis.as_bytes());
    TruthDeltaSurfaceIdentity::admit_bridge_owned(format!("truth-delta-surface:sha256:{digest:x}"))
}

fn truth_delta_surface_target_mask_identity(
    target: &BridgeCommittedPatchTarget,
) -> TruthDeltaSurfaceTargetMaskIdentity {
    let basis = format!(
        "truth-delta-surface-target-mask|target={}",
        target.canonical_basis()
    );
    let digest = Sha256::digest(basis.as_bytes());
    TruthDeltaSurfaceTargetMaskIdentity::admit_bridge_owned(format!(
        "truth-delta-surface-target-mask:sha256:{digest:x}"
    ))
}

impl TruthPatchTargetView for TruthDeltaSurface {
    fn truth_surface_kind(&self) -> TruthDeltaSurfaceKind {
        self.surface_kind()
    }

    fn truth_field_path(&self) -> Option<&worth_foundational::facade::CanonicalFieldPath> {
        self.field_locator().map(AspectFieldLocator::field_path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTruthDeltaSurfaceSet {
    surfaces: Vec<TruthDeltaSurface>,
    item_surfaces: Vec<TruthDeltaSurface>,
}

impl NormalizedTruthDeltaSurfaceSet {
    fn new(surfaces: Vec<TruthDeltaSurface>, item_surfaces: Vec<TruthDeltaSurface>) -> Self {
        Self {
            surfaces,
            item_surfaces,
        }
    }

    pub(crate) fn item_surfaces(&self) -> &[TruthDeltaSurface] {
        &self.item_surfaces
    }

    pub fn len(&self) -> usize {
        self.surfaces.len()
    }
}

pub(crate) fn derive_normalized_truth_delta_surface_set(
    envelope: &BridgeCommittedPatchEnvelope,
) -> NormalizedTruthDeltaSurfaceSet {
    let mut item_surfaces = Vec::with_capacity(envelope.patch_body().canonical_items().len());
    for item in envelope.patch_body().canonical_items() {
        item_surfaces.push(derive_surface(item));
    }

    let mut surfaces = item_surfaces.clone();
    surfaces.sort();
    surfaces.dedup();
    NormalizedTruthDeltaSurfaceSet::new(surfaces, item_surfaces)
}

pub(crate) fn truth_delta_surface_count(envelope: &BridgeCommittedPatchEnvelope) -> usize {
    envelope.patch_body().canonical_items().len()
}

fn derive_surface(item: &BridgeCommittedPatchItem) -> TruthDeltaSurface {
    TruthDeltaSurface::from_native_target_parts(
        item.entity_identity(),
        item.relational_record_identity_parts(),
        item.target().clone(),
        Arc::from(item.target_canonical_basis()),
    )
}

pub(crate) fn canonical_truth_surface_kind_label(
    surface_kind: TruthDeltaSurfaceKind,
) -> &'static str {
    match surface_kind {
        TruthDeltaSurfaceKind::AuthoritativeAspect => "authoritative-aspect",
        TruthDeltaSurfaceKind::EntityField => "entity-field",
        TruthDeltaSurfaceKind::EntityRelationEndpoint => "entity-relation-endpoint",
        TruthDeltaSurfaceKind::EntityRegion => "entity-region",
        TruthDeltaSurfaceKind::EntityPartition => "entity-partition",
        TruthDeltaSurfaceKind::EntityFacet => "entity-facet",
        TruthDeltaSurfaceKind::LifecycleTransition => "lifecycle-transition",
    }
}

impl FrozenAspectMappingRegistry {
    pub(crate) fn lookup(
        &self,
        entity_identity: &str,
        aspect_key: &AspectKey,
        field_path: Option<&worth_foundational::facade::CanonicalFieldPath>,
        surface_kind: TruthDeltaSurfaceKind,
    ) -> Option<&crate::mapping::aspects::FrozenAspectRegistration> {
        self.registrations().iter().find(|registration| {
            registration.truth_surface_kind() == surface_kind
                && registration
                    .truth_scope()
                    .entity_selector()
                    .matches(entity_identity)
                && registration
                    .truth_scope()
                    .aspect_selector()
                    .matches(aspect_key)
                && registration
                    .truth_scope()
                    .target_selector()
                    .matches(&AspectTargetView {
                        surface_kind,
                        field_path,
                    })
        })
    }
}

struct AspectTargetView<'a> {
    surface_kind: TruthDeltaSurfaceKind,
    field_path: Option<&'a worth_foundational::facade::CanonicalFieldPath>,
}

impl TruthPatchTargetView for AspectTargetView<'_> {
    fn truth_surface_kind(&self) -> TruthDeltaSurfaceKind {
        self.surface_kind
    }

    fn truth_field_path(&self) -> Option<&worth_foundational::facade::CanonicalFieldPath> {
        self.field_path
    }
}

#[cfg(test)]
#[path = "surfaces_tests.rs"]
mod surfaces_tests;
