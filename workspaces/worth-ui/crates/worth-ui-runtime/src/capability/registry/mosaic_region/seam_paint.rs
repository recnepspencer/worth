use std::collections::BTreeSet;

use crate::capability::MosaicRegionKindId;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MosaicSharedEdge {
    first: MosaicRegionKindId,
    second: MosaicRegionKindId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MosaicSeamPaintOwner {
    edge: MosaicSharedEdge,
    owner: MosaicRegionKindId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MosaicExteriorCornerPosture {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MosaicExteriorCorner {
    region: MosaicRegionKindId,
    posture: MosaicExteriorCornerPosture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MosaicSeamPaintContract {
    regions: Box<[MosaicRegionKindId]>,
    shared_edges: Box<[MosaicSharedEdge]>,
    owners: Box<[MosaicSeamPaintOwner]>,
    exterior_corners: Box<[MosaicExteriorCorner]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MosaicSeamPaintContractDenial {
    EmptyRegions,
    DuplicateContract,
    DuplicateRegion(MosaicRegionKindId),
    ForeignEndpoint(MosaicRegionKindId),
    SelfEdge,
    OwnerIsNotEndpoint,
    DuplicateSharedEdge,
    DuplicateOwner,
    MissingOwner(MosaicSharedEdge),
    ForeignOwnerEdge(MosaicSharedEdge),
    DuplicateExteriorCorner,
}

impl MosaicSharedEdge {
    pub fn new(
        first: MosaicRegionKindId,
        second: MosaicRegionKindId,
    ) -> Result<Self, MosaicSeamPaintContractDenial> {
        if first == second {
            return Err(MosaicSeamPaintContractDenial::SelfEdge);
        }
        Ok(if first < second {
            Self { first, second }
        } else {
            Self {
                first: second,
                second: first,
            }
        })
    }
    pub fn endpoints(&self) -> [&MosaicRegionKindId; 2] {
        [&self.first, &self.second]
    }
}

impl MosaicSeamPaintOwner {
    pub fn new(
        edge: MosaicSharedEdge,
        owner: MosaicRegionKindId,
    ) -> Result<Self, MosaicSeamPaintContractDenial> {
        if owner != edge.first && owner != edge.second {
            return Err(MosaicSeamPaintContractDenial::OwnerIsNotEndpoint);
        }
        Ok(Self { edge, owner })
    }
    pub const fn edge(&self) -> &MosaicSharedEdge {
        &self.edge
    }
    pub const fn owner(&self) -> &MosaicRegionKindId {
        &self.owner
    }
}

impl MosaicExteriorCorner {
    pub const fn new(region: MosaicRegionKindId, posture: MosaicExteriorCornerPosture) -> Self {
        Self { region, posture }
    }
    pub const fn region(&self) -> &MosaicRegionKindId {
        &self.region
    }
    pub const fn posture(&self) -> MosaicExteriorCornerPosture {
        self.posture
    }
}

impl MosaicSeamPaintContract {
    pub fn admit(
        regions: impl IntoIterator<Item = MosaicRegionKindId>,
        shared_edges: impl IntoIterator<Item = MosaicSharedEdge>,
        owners: impl IntoIterator<Item = MosaicSeamPaintOwner>,
        exterior_corners: impl IntoIterator<Item = MosaicExteriorCorner>,
    ) -> Result<Self, MosaicSeamPaintContractDenial> {
        let mut regions = regions.into_iter().collect::<Vec<_>>();
        regions.sort();
        if regions.is_empty() {
            return Err(MosaicSeamPaintContractDenial::EmptyRegions);
        }
        if let Some(pair) = regions.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(MosaicSeamPaintContractDenial::DuplicateRegion(
                pair[0].clone(),
            ));
        }
        let region_set = regions.iter().cloned().collect::<BTreeSet<_>>();
        let mut shared_edges = shared_edges.into_iter().collect::<Vec<_>>();
        validate_edges(&region_set, &shared_edges)?;
        shared_edges.sort();
        if shared_edges.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(MosaicSeamPaintContractDenial::DuplicateSharedEdge);
        }
        let mut owners = owners.into_iter().collect::<Vec<_>>();
        owners.sort_by(|left, right| left.edge.cmp(&right.edge));
        if owners.windows(2).any(|pair| pair[0].edge == pair[1].edge) {
            return Err(MosaicSeamPaintContractDenial::DuplicateOwner);
        }
        for edge in &shared_edges {
            if !owners.iter().any(|owner| &owner.edge == edge) {
                return Err(MosaicSeamPaintContractDenial::MissingOwner(edge.clone()));
            }
        }
        for owner in &owners {
            if shared_edges.binary_search(&owner.edge).is_err() {
                return Err(MosaicSeamPaintContractDenial::ForeignOwnerEdge(
                    owner.edge.clone(),
                ));
            }
        }
        let mut exterior_corners = exterior_corners.into_iter().collect::<Vec<_>>();
        for corner in &exterior_corners {
            if !region_set.contains(&corner.region) {
                return Err(MosaicSeamPaintContractDenial::ForeignEndpoint(
                    corner.region.clone(),
                ));
            }
        }
        exterior_corners.sort();
        if exterior_corners.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(MosaicSeamPaintContractDenial::DuplicateExteriorCorner);
        }
        Ok(Self {
            regions: regions.into_boxed_slice(),
            shared_edges: shared_edges.into_boxed_slice(),
            owners: owners.into_boxed_slice(),
            exterior_corners: exterior_corners.into_boxed_slice(),
        })
    }
    pub fn regions(&self) -> &[MosaicRegionKindId] {
        &self.regions
    }
    pub fn shared_edges(&self) -> &[MosaicSharedEdge] {
        &self.shared_edges
    }
    pub fn owners(&self) -> &[MosaicSeamPaintOwner] {
        &self.owners
    }
    pub fn paint_owner_for(
        &self,
        first: &MosaicRegionKindId,
        second: &MosaicRegionKindId,
    ) -> Option<&MosaicRegionKindId> {
        let edge = MosaicSharedEdge::new(first.clone(), second.clone()).ok()?;
        self.owners
            .binary_search_by(|candidate| candidate.edge.cmp(&edge))
            .ok()
            .map(|index| self.owners[index].owner())
    }
    pub fn exterior_corners(&self) -> &[MosaicExteriorCorner] {
        &self.exterior_corners
    }
}

fn validate_edges(
    regions: &BTreeSet<MosaicRegionKindId>,
    edges: &[MosaicSharedEdge],
) -> Result<(), MosaicSeamPaintContractDenial> {
    for edge in edges {
        for endpoint in edge.endpoints() {
            if !regions.contains(endpoint) {
                return Err(MosaicSeamPaintContractDenial::ForeignEndpoint(
                    endpoint.clone(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{MosaicRegionKindDescriptor, MosaicRegionRole};

    fn region(name: &str) -> MosaicRegionKindId {
        MosaicRegionKindId::new(name).unwrap()
    }

    fn admitted_region(
        id: MosaicRegionKindId,
        role: MosaicRegionRole,
    ) -> MosaicRegionKindDescriptor {
        MosaicRegionKindDescriptor::new(id, role)
            .with_sizing_behavior(crate::capability::MosaicSizingBehavior::fills_available_space())
            .with_scroll_ownership(crate::capability::MosaicScrollOwnership::region_owned())
            .with_focus_scope(crate::capability::MosaicFocusScopeKind::active_surface_scope())
            .with_child_rule(crate::capability::MosaicChildRule::accepts_surfaces())
            .with_allowed_surface_class(crate::capability::SurfacePlacementClass::primary_region())
            .with_persistence(crate::capability::MosaicRegionPersistence::restorable())
            .with_clipping(crate::capability::MosaicClippingPosture::clip_to_region())
            .with_hit_test(crate::capability::MosaicHitTestPosture::participates())
    }

    #[test]
    fn exact_shared_edge_partition_requires_one_endpoint_owner() {
        let a = region("region.a");
        let b = region("region.b");
        let edge = MosaicSharedEdge::new(a.clone(), b.clone()).unwrap();
        assert_eq!(
            MosaicSeamPaintContract::admit([a.clone(), b.clone()], [edge.clone()], [], []),
            Err(MosaicSeamPaintContractDenial::MissingOwner(edge.clone()))
        );
        let owner = MosaicSeamPaintOwner::new(edge.clone(), a.clone()).unwrap();
        let contract =
            MosaicSeamPaintContract::admit([a.clone(), b.clone()], [edge], [owner], []).unwrap();
        assert_eq!(contract.paint_owner_for(&a, &b), Some(&a));
        assert_eq!(contract.paint_owner_for(&b, &a), Some(&a));
    }

    #[test]
    fn frozen_snapshot_reports_the_seam_family_independently() {
        let a = region("region.a");
        let b = region("region.b");
        let contract = MosaicSeamPaintContract::admit([a.clone(), b.clone()], [], [], []).unwrap();
        let snapshot = crate::facade::entry::CapabilityRegistrationBuilder::new()
            .register_mosaic_region_kind(admitted_region(a, MosaicRegionRole::primary()))
            .register_mosaic_region_kind(admitted_region(b, MosaicRegionRole::auxiliary()))
            .register_mosaic_seam_paint_contract(contract)
            .unwrap()
            .freeze_with_registration_report()
            .into_accepted_snapshot();

        assert_eq!(
            snapshot
                .freeze_report()
                .registry_family_width(crate::capability::RegistryFamily::MosaicSeamPaint),
            Some(1)
        );
        assert!(snapshot
            .freeze_report()
            .has_complete_registry_family_inventory());
    }
}
