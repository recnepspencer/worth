use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::{
    UiBackdropDeclaration, UiBackdropIdentity, UiMosaicRegionDeclarationIdentity,
    UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayDependencyKind {
    Scope,
    Presence,
    Placement,
    Motion,
    Extent,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayChangedBasis {
    Portal(UiPortalDeclarationId),
    PortalScope(UiPortalDeclarationId),
    PortalPresence(UiPortalDeclarationId),
    PortalPlacement(UiPortalDeclarationId),
    PortalMotion(UiPortalDeclarationId),
    Backdrop(UiBackdropIdentity),
    SurfaceExtent(UiSemanticSurfaceDeclarationIdentity),
    RegionExtent(UiMosaicRegionDeclarationIdentity),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiOverlayChangeSet {
    changes: Box<[UiOverlayChangedBasis]>,
}

impl UiOverlayChangeSet {
    pub(crate) fn from_changes(changes: impl IntoIterator<Item = UiOverlayChangedBasis>) -> Self {
        let mut changes = changes.into_iter().collect::<Vec<_>>();
        changes.sort_unstable();
        changes.dedup();
        Self {
            changes: changes.into_boxed_slice(),
        }
    }

    pub(crate) fn changes(&self) -> &[UiOverlayChangedBasis] {
        &self.changes
    }

    pub(crate) fn has_structural_change(&self) -> bool {
        self.changes.iter().any(|change| {
            matches!(
                change,
                UiOverlayChangedBasis::Portal(_)
                    | UiOverlayChangedBasis::PortalScope(_)
                    | UiOverlayChangedBasis::PortalPresence(_)
                    | UiOverlayChangedBasis::PortalPlacement(_)
                    | UiOverlayChangedBasis::Backdrop(_)
            )
        })
    }

    pub(crate) fn has_motion_change(&self) -> bool {
        self.changes
            .iter()
            .any(|change| matches!(change, UiOverlayChangedBasis::PortalMotion(_)))
    }

    pub(crate) fn has_extent_change(&self) -> bool {
        self.changes.iter().any(|change| {
            matches!(
                change,
                UiOverlayChangedBasis::SurfaceExtent(_) | UiOverlayChangedBasis::RegionExtent(_)
            )
        })
    }

    pub(crate) fn has_declaration_change(&self) -> bool {
        self.changes
            .iter()
            .any(|change| matches!(change, UiOverlayChangedBasis::Backdrop(_)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayAffectedBackdrop {
    identity: UiBackdropIdentity,
    reasons: Box<[UiOverlayDependencyKind]>,
}

impl UiOverlayAffectedBackdrop {
    pub(crate) const fn identity(&self) -> UiBackdropIdentity {
        self.identity
    }

    pub(crate) fn reasons(&self) -> &[UiOverlayDependencyKind] {
        &self.reasons
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayAffectedScope {
    backdrops: Box<[UiOverlayAffectedBackdrop]>,
    portals: Box<[UiPortalDeclarationId]>,
    surfaces: Box<[UiSemanticSurfaceDeclarationIdentity]>,
    regions: Box<[UiMosaicRegionDeclarationIdentity]>,
}

impl UiOverlayAffectedScope {
    pub(crate) fn backdrops(&self) -> &[UiOverlayAffectedBackdrop] {
        &self.backdrops
    }

    pub(crate) fn portals(&self) -> &[UiPortalDeclarationId] {
        &self.portals
    }

    pub(crate) fn surfaces(&self) -> &[UiSemanticSurfaceDeclarationIdentity] {
        &self.surfaces
    }

    pub(crate) fn regions(&self) -> &[UiMosaicRegionDeclarationIdentity] {
        &self.regions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayDependencyIndex {
    by_identity: BTreeMap<UiBackdropIdentity, usize>,
    by_scope: BTreeMap<UiPortalDeclarationId, Box<[UiBackdropIdentity]>>,
    by_presence: BTreeMap<UiPortalDeclarationId, Box<[UiBackdropIdentity]>>,
    by_placement_portal: BTreeMap<UiPortalDeclarationId, Box<[UiBackdropIdentity]>>,
    by_motion: BTreeMap<UiPortalDeclarationId, Box<[UiBackdropIdentity]>>,
    by_relation_backdrop: BTreeMap<UiBackdropIdentity, Box<[UiBackdropIdentity]>>,
    by_surface_extent: BTreeMap<UiSemanticSurfaceDeclarationIdentity, Box<[UiBackdropIdentity]>>,
    by_region_extent: BTreeMap<UiMosaicRegionDeclarationIdentity, Box<[UiBackdropIdentity]>>,
}

impl UiOverlayDependencyIndex {
    pub(crate) fn rebuild(declarations: &[UiBackdropDeclaration]) -> Result<Self, ()> {
        let mut by_identity = BTreeMap::new();
        let mut by_scope = BTreeMap::<UiPortalDeclarationId, Vec<UiBackdropIdentity>>::new();
        let mut by_presence = BTreeMap::<UiPortalDeclarationId, Vec<UiBackdropIdentity>>::new();
        let mut by_placement_portal =
            BTreeMap::<UiPortalDeclarationId, Vec<UiBackdropIdentity>>::new();
        let mut by_motion = BTreeMap::<UiPortalDeclarationId, Vec<UiBackdropIdentity>>::new();
        let mut by_relation_backdrop =
            BTreeMap::<UiBackdropIdentity, Vec<UiBackdropIdentity>>::new();
        let mut by_surface_extent =
            BTreeMap::<UiSemanticSurfaceDeclarationIdentity, Vec<UiBackdropIdentity>>::new();
        let mut by_region_extent =
            BTreeMap::<UiMosaicRegionDeclarationIdentity, Vec<UiBackdropIdentity>>::new();

        for (index, declaration) in declarations.iter().enumerate() {
            let identity = declaration.identity();
            if by_identity.insert(identity, index).is_some() {
                return Err(());
            }
            if let worth_ui_dsl::UiBackdropScope::PerPortalInstance(portal) = declaration.scope() {
                by_scope.entry(portal).or_default().push(identity);
            }
            if let worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(portal) =
                declaration.presence()
            {
                by_presence.entry(portal).or_default().push(identity);
            }
            if let Some(portal) = declaration.placement().portal_anchor() {
                by_placement_portal
                    .entry(portal)
                    .or_default()
                    .push(identity);
            }
            if let worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(portal) =
                declaration.motion()
            {
                by_motion.entry(portal).or_default().push(identity);
            }
            match declaration.extent() {
                worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(surface) => {
                    by_surface_extent.entry(surface).or_default().push(identity);
                }
                worth_ui_dsl::UiBackdropExtentBasis::PresentedMosaicRegion { region, .. } => {
                    by_region_extent.entry(region).or_default().push(identity);
                }
            }
            if let worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforeBackdrop(target)
            | worth_ui_dsl::UiBackdropPlacement::ImmediatelyAfterBackdrop(target) =
                declaration.placement()
            {
                by_relation_backdrop
                    .entry(target)
                    .or_default()
                    .push(identity);
                by_relation_backdrop
                    .entry(identity)
                    .or_default()
                    .push(target);
            }
        }

        Ok(Self {
            by_identity,
            by_scope: boxed(by_scope),
            by_presence: boxed(by_presence),
            by_placement_portal: boxed(by_placement_portal),
            by_motion: boxed(by_motion),
            by_relation_backdrop: boxed(by_relation_backdrop),
            by_surface_extent: boxed(by_surface_extent),
            by_region_extent: boxed(by_region_extent),
        })
    }

    pub(crate) fn declaration_index(&self, identity: UiBackdropIdentity) -> Option<usize> {
        self.by_identity.get(&identity).copied()
    }

    pub(crate) fn scope_dependents(&self, portal: UiPortalDeclarationId) -> &[UiBackdropIdentity] {
        self.by_scope.get(&portal).map_or(&[], Box::as_ref)
    }

    pub(crate) fn presence_dependents(
        &self,
        portal: UiPortalDeclarationId,
    ) -> &[UiBackdropIdentity] {
        self.by_presence.get(&portal).map_or(&[], Box::as_ref)
    }

    pub(crate) fn placement_dependents(
        &self,
        portal: UiPortalDeclarationId,
    ) -> &[UiBackdropIdentity] {
        self.by_placement_portal
            .get(&portal)
            .map_or(&[], Box::as_ref)
    }

    pub(crate) fn motion_dependents(&self, portal: UiPortalDeclarationId) -> &[UiBackdropIdentity] {
        self.by_motion.get(&portal).map_or(&[], Box::as_ref)
    }

    pub(crate) fn relation_dependents(
        &self,
        backdrop: UiBackdropIdentity,
    ) -> &[UiBackdropIdentity] {
        self.by_relation_backdrop
            .get(&backdrop)
            .map_or(&[], Box::as_ref)
    }

    pub(crate) fn affected_scope(&self, changes: &UiOverlayChangeSet) -> UiOverlayAffectedScope {
        let mut selected = BTreeMap::<UiBackdropIdentity, BTreeSet<UiOverlayDependencyKind>>::new();
        let mut portals = BTreeSet::new();
        let mut surfaces = BTreeSet::new();
        let mut regions = BTreeSet::new();
        let close_relations = changes.changes().iter().any(|change| {
            matches!(
                change,
                UiOverlayChangedBasis::Portal(_)
                    | UiOverlayChangedBasis::PortalScope(_)
                    | UiOverlayChangedBasis::PortalPresence(_)
                    | UiOverlayChangedBasis::PortalPlacement(_)
                    | UiOverlayChangedBasis::Backdrop(_)
            )
        });
        for change in changes.changes() {
            match *change {
                UiOverlayChangedBasis::Portal(portal) => {
                    portals.insert(portal);
                    add(
                        &mut selected,
                        self.scope_dependents(portal),
                        UiOverlayDependencyKind::Scope,
                    );
                    add(
                        &mut selected,
                        self.presence_dependents(portal),
                        UiOverlayDependencyKind::Presence,
                    );
                    add(
                        &mut selected,
                        self.placement_dependents(portal),
                        UiOverlayDependencyKind::Placement,
                    );
                    add(
                        &mut selected,
                        self.motion_dependents(portal),
                        UiOverlayDependencyKind::Motion,
                    );
                }
                UiOverlayChangedBasis::PortalScope(portal) => {
                    portals.insert(portal);
                    add(
                        &mut selected,
                        self.scope_dependents(portal),
                        UiOverlayDependencyKind::Scope,
                    );
                }
                UiOverlayChangedBasis::PortalPresence(portal) => {
                    portals.insert(portal);
                    add(
                        &mut selected,
                        self.presence_dependents(portal),
                        UiOverlayDependencyKind::Presence,
                    );
                }
                UiOverlayChangedBasis::PortalPlacement(portal) => {
                    portals.insert(portal);
                    add(
                        &mut selected,
                        self.placement_dependents(portal),
                        UiOverlayDependencyKind::Placement,
                    );
                }
                UiOverlayChangedBasis::PortalMotion(portal) => {
                    portals.insert(portal);
                    add(
                        &mut selected,
                        self.motion_dependents(portal),
                        UiOverlayDependencyKind::Motion,
                    );
                }
                UiOverlayChangedBasis::Backdrop(identity) => {
                    add(
                        &mut selected,
                        &[identity],
                        UiOverlayDependencyKind::Placement,
                    );
                }
                UiOverlayChangedBasis::SurfaceExtent(surface) => {
                    surfaces.insert(surface);
                    add(
                        &mut selected,
                        self.by_surface_extent
                            .get(&surface)
                            .map_or(&[], Box::as_ref),
                        UiOverlayDependencyKind::Extent,
                    );
                }
                UiOverlayChangedBasis::RegionExtent(region) => {
                    regions.insert(region);
                    add(
                        &mut selected,
                        self.by_region_extent.get(&region).map_or(&[], Box::as_ref),
                        UiOverlayDependencyKind::Extent,
                    );
                }
            }
        }
        if close_relations {
            let mut pending = selected.keys().copied().collect::<Vec<_>>();
            let mut visited = BTreeSet::new();
            while let Some(identity) = pending.pop() {
                if !visited.insert(identity) {
                    continue;
                }
                for dependent in self.relation_dependents(identity) {
                    selected
                        .entry(*dependent)
                        .or_default()
                        .insert(UiOverlayDependencyKind::Placement);
                    pending.push(*dependent);
                }
            }
        }
        let backdrops = selected
            .into_iter()
            .map(|(identity, reasons)| UiOverlayAffectedBackdrop {
                identity,
                reasons: reasons.into_iter().collect(),
            })
            .collect();
        UiOverlayAffectedScope {
            backdrops,
            portals: portals.into_iter().collect(),
            surfaces: surfaces.into_iter().collect(),
            regions: regions.into_iter().collect(),
        }
    }
}

fn add(
    selected: &mut BTreeMap<UiBackdropIdentity, BTreeSet<UiOverlayDependencyKind>>,
    identities: &[UiBackdropIdentity],
    reason: UiOverlayDependencyKind,
) {
    for identity in identities {
        selected.entry(*identity).or_default().insert(reason);
    }
}

fn boxed<K: Ord>(
    values: BTreeMap<K, Vec<UiBackdropIdentity>>,
) -> BTreeMap<K, Box<[UiBackdropIdentity]>> {
    values
        .into_iter()
        .map(|(key, mut values)| {
            values.sort_unstable();
            values.dedup();
            (key, values.into_boxed_slice())
        })
        .collect()
}
