use std::collections::{BTreeMap, BTreeSet};

/// Compiler-issued declaration identities retained with one sealed semantic
/// package. The maps are private so consumers can resolve authored names
/// without mutating or replacing compiler authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSealedOverlayDeclarationBindings {
    backdrops: BTreeMap<String, crate::UiBackdropIdentity>,
    portals: BTreeMap<String, crate::UiPortalDeclarationId>,
    surfaces: BTreeMap<String, crate::UiSemanticSurfaceDeclarationIdentity>,
    regions: BTreeMap<String, crate::UiMosaicRegionDeclarationIdentity>,
}

impl WorthUiSealedOverlayDeclarationBindings {
    pub(super) fn from_parts(
        backdrops: BTreeMap<String, crate::UiBackdropIdentity>,
        portals: BTreeMap<String, crate::UiPortalDeclarationId>,
        surfaces: BTreeMap<String, crate::UiSemanticSurfaceDeclarationIdentity>,
        regions: BTreeMap<String, crate::UiMosaicRegionDeclarationIdentity>,
    ) -> Self {
        Self {
            backdrops,
            portals,
            surfaces,
            regions,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.backdrops.is_empty()
            && self.portals.is_empty()
            && self.surfaces.is_empty()
            && self.regions.is_empty()
    }

    pub fn backdrop_named(&self, name: &str) -> Option<crate::UiBackdropIdentity> {
        self.backdrops.get(name).copied()
    }

    pub fn portal_named(&self, name: &str) -> Option<crate::UiPortalDeclarationId> {
        self.portals.get(name).copied()
    }

    pub fn surface_named(&self, name: &str) -> Option<crate::UiSemanticSurfaceDeclarationIdentity> {
        self.surfaces.get(name).copied()
    }

    pub fn contains_surface(&self, identity: crate::UiSemanticSurfaceDeclarationIdentity) -> bool {
        self.surfaces
            .values()
            .any(|candidate| *candidate == identity)
    }

    pub fn region_named(
        &self,
        surface: &str,
        region: &str,
    ) -> Option<crate::UiMosaicRegionDeclarationIdentity> {
        self.regions.get(&region_key(surface, region)).copied()
    }

    pub fn region_on_surface(
        &self,
        surface: crate::UiSemanticSurfaceDeclarationIdentity,
        region: &str,
    ) -> Option<crate::UiMosaicRegionDeclarationIdentity> {
        let surface_name = self
            .surfaces
            .iter()
            .find_map(|(name, identity)| (*identity == surface).then_some(name.as_str()))?;
        self.region_named(surface_name, region)
    }
}

pub(super) fn allocate<T>(
    names: BTreeSet<String>,
    constructor: impl Fn(u64) -> Option<T>,
) -> Result<BTreeMap<String, T>, ()> {
    names
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let value = index
                .checked_add(1)
                .and_then(|value| u64::try_from(value).ok())
                .ok_or(())?;
            let identity = constructor(value).ok_or(())?;
            Ok((name, identity))
        })
        .collect()
}

pub(super) fn region_key(surface: &str, region: &str) -> String {
    let mut key = surface.to_owned();
    key.push(char::from(0));
    key.push_str(region);
    key
}
