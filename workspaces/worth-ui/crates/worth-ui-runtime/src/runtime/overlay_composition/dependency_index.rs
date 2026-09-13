use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::{
    UiBackdropDeclaration, UiBackdropIdentity, UiMosaicRegionDeclarationIdentity,
    UiPortalDeclarationId, UiSemanticSurfaceDeclarationIdentity,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum UiOverlayChangedBasis {
    Portal(UiPortalDeclarationId),
    PortalMotion(UiPortalDeclarationId),
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
        self.changes
            .iter()
            .any(|change| matches!(change, UiOverlayChangedBasis::Portal(_)))
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayAffectedBackdrop {
    identity: UiBackdropIdentity,
}

impl UiOverlayAffectedBackdrop {
    pub(crate) const fn identity(&self) -> UiBackdropIdentity {
        self.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayAffectedScope {
    backdrops: Box<[UiOverlayAffectedBackdrop]>,
}

impl UiOverlayAffectedScope {
    pub(crate) fn backdrops(&self) -> &[UiOverlayAffectedBackdrop] {
        &self.backdrops
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
        let mut selected = BTreeSet::<UiBackdropIdentity>::new();
        let close_relations = changes
            .changes()
            .iter()
            .any(|change| matches!(change, UiOverlayChangedBasis::Portal(_)));
        for change in changes.changes() {
            match *change {
                UiOverlayChangedBasis::Portal(portal) => {
                    add(&mut selected, self.scope_dependents(portal));
                    add(&mut selected, self.presence_dependents(portal));
                    add(&mut selected, self.placement_dependents(portal));
                    add(&mut selected, self.motion_dependents(portal));
                }
                UiOverlayChangedBasis::PortalMotion(portal) => {
                    add(&mut selected, self.motion_dependents(portal));
                }
                UiOverlayChangedBasis::SurfaceExtent(surface) => {
                    add(
                        &mut selected,
                        self.by_surface_extent
                            .get(&surface)
                            .map_or(&[], Box::as_ref),
                    );
                }
                UiOverlayChangedBasis::RegionExtent(region) => {
                    add(
                        &mut selected,
                        self.by_region_extent.get(&region).map_or(&[], Box::as_ref),
                    );
                }
            }
        }
        if close_relations {
            let mut pending = selected.iter().copied().collect::<Vec<_>>();
            let mut visited = BTreeSet::new();
            while let Some(identity) = pending.pop() {
                if !visited.insert(identity) {
                    continue;
                }
                for dependent in self.relation_dependents(identity) {
                    selected.insert(*dependent);
                    pending.push(*dependent);
                }
            }
        }
        let backdrops = selected
            .into_iter()
            .map(|identity| UiOverlayAffectedBackdrop { identity })
            .collect();
        UiOverlayAffectedScope { backdrops }
    }
}

fn add(selected: &mut BTreeSet<UiBackdropIdentity>, identities: &[UiBackdropIdentity]) {
    for identity in identities {
        selected.insert(*identity);
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
