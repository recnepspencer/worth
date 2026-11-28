use std::borrow::Cow;

use crate::input::envelope::BridgeCommittedPatchItem;
use crate::mapping::freezing::{FrozenBridgeMappingRegistration, FrozenMappingRegistry};
use crate::mapping::{TruthDeltaSurfaceKind, TruthPatchTargetView};
use crate::routing::surfaces::TruthDeltaSurface;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMappingLookupKey<'a> {
    entity_identity: Cow<'a, str>,
    aspect_key: &'a AspectKey,
    field_path: Option<&'a CanonicalFieldPath>,
    surface_kind: TruthDeltaSurfaceKind,
}

impl<'a> BridgeMappingLookupKey<'a> {
    pub fn new(
        entity_identity: &'a str,
        aspect_key: &'a AspectKey,
        field_path: Option<&'a CanonicalFieldPath>,
        surface_kind: TruthDeltaSurfaceKind,
    ) -> Self {
        Self {
            entity_identity: Cow::Borrowed(entity_identity),
            aspect_key,
            field_path,
            surface_kind,
        }
    }

    pub fn entity_identity(&self) -> &str {
        self.entity_identity.as_ref()
    }

    pub fn aspect_key(&self) -> &AspectKey {
        self.aspect_key
    }
}

impl<'a> From<&'a BridgeCommittedPatchItem> for BridgeMappingLookupKey<'a> {
    fn from(value: &'a BridgeCommittedPatchItem) -> Self {
        Self {
            entity_identity: value.entity_identity(),
            aspect_key: value.aspect_key(),
            field_path: value.field_locator().map(|locator| locator.field_path()),
            surface_kind: value.surface_kind(),
        }
    }
}

impl TruthPatchTargetView for BridgeMappingLookupKey<'_> {
    fn truth_surface_kind(&self) -> TruthDeltaSurfaceKind {
        self.surface_kind
    }

    fn truth_field_path(&self) -> Option<&CanonicalFieldPath> {
        self.field_path
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedBridgeMapping<'a> {
    registration: &'a FrozenBridgeMappingRegistration,
}

impl<'a> ResolvedBridgeMapping<'a> {
    pub(crate) fn new(registration: &'a FrozenBridgeMappingRegistration) -> Self {
        Self { registration }
    }

    pub(crate) fn registration(&self) -> &'a FrozenBridgeMappingRegistration {
        self.registration
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBridgeMappings<'a> {
    resolved: Vec<ResolvedBridgeMapping<'a>>,
}

impl<'a> ResolvedBridgeMappings<'a> {
    pub(crate) fn new(resolved: Vec<ResolvedBridgeMapping<'a>>) -> Self {
        Self { resolved }
    }

    pub(crate) fn registrations(
        &self,
    ) -> impl Iterator<Item = &'a FrozenBridgeMappingRegistration> + '_ {
        self.resolved
            .iter()
            .map(ResolvedBridgeMapping::registration)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeMappingLookup<'a> {
    Exact {
        resolved: ResolvedBridgeMappings<'a>,
    },
    Widening {
        resolved: ResolvedBridgeMappings<'a>,
    },
    Missing,
}

impl FrozenMappingRegistry {
    pub(crate) fn lookup<'a>(&'a self, key: BridgeMappingLookupKey<'_>) -> BridgeMappingLookup<'a> {
        let mut matches = self
            .registrations
            .iter()
            .filter(|registration| registration.truth_scope().matches_key(&key))
            .collect::<Vec<_>>();
        if matches.is_empty() {
            return BridgeMappingLookup::Missing;
        }

        let most_specific_rank = matches
            .iter()
            .map(|registration| registration.truth_scope().specificity_rank())
            .max()
            .expect("non-empty matching registrations should have a specificity rank");
        matches.retain(|registration| {
            registration.truth_scope().specificity_rank() == most_specific_rank
        });

        let is_widening = matches
            .first()
            .and_then(|registration| registration.widening_class())
            .is_some();
        let resolved = ResolvedBridgeMappings::new(
            matches
                .into_iter()
                .map(ResolvedBridgeMapping::new)
                .collect(),
        );
        match is_widening {
            false => BridgeMappingLookup::Exact { resolved },
            true => BridgeMappingLookup::Widening { resolved },
        }
    }
    pub(crate) fn lookup_truth_surface<'a>(
        &'a self,
        surface: &'a TruthDeltaSurface,
    ) -> BridgeMappingLookup<'a> {
        self.lookup(BridgeMappingLookupKey::new(
            surface.entity_identity().as_ref(),
            surface.aspect_key(),
            surface.field_locator().map(|locator| locator.field_path()),
            surface.surface_kind(),
        ))
    }
}
