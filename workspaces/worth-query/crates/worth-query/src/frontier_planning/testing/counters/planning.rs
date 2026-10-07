use crate::identity::PlanDigest;

use super::super::{
    BundleResolvedBasisDigest, PacketEquivalenceContract, PacketMergeBoundary,
    PlannedWorkPacketDigest,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum FrontierPlanFamily {
    OrderedCollection,
    BoundedMaterialization,
    LiveDetail,
    LiveOrderedCollection,
    LiveBoundedMaterialization,
}

impl FrontierPlanFamily {
    pub(in crate::frontier_planning::testing) fn as_str(&self) -> &'static str {
        match self {
            Self::OrderedCollection => "ordered_collection",
            Self::BoundedMaterialization => "bounded_materialization",
            Self::LiveDetail => "live_detail",
            Self::LiveOrderedCollection => "live_ordered_collection",
            Self::LiveBoundedMaterialization => "live_bounded_materialization",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum PlannedWorkPacketFamily {
    OrderedCollectionRoot,
    BoundedMaterializationRoot,
    LiveDetailRoot,
    LiveOrderedCollectionRoot,
    LiveBoundedMaterializationRoot,
}

impl PlannedWorkPacketFamily {
    fn as_str(&self) -> &'static str {
        match self {
            Self::OrderedCollectionRoot => "ordered_collection_root",
            Self::BoundedMaterializationRoot => "bounded_materialization_root",
            Self::LiveDetailRoot => "live_detail_root",
            Self::LiveOrderedCollectionRoot => "live_ordered_collection_root",
            Self::LiveBoundedMaterializationRoot => "live_bounded_materialization_root",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlannedWorkPacket {
    source_plan_digest: PlanDigest,
    family: PlannedWorkPacketFamily,
    ordinal: usize,
    digest: PlannedWorkPacketDigest,
    scope_summary: String,
    merge_boundary: PacketMergeBoundary,
}

impl PlannedWorkPacket {
    pub(crate) fn family(&self) -> &PlannedWorkPacketFamily {
        &self.family
    }

    pub(crate) fn digest(&self) -> &PlannedWorkPacketDigest {
        &self.digest
    }

    pub(crate) fn merge_boundary(&self) -> &PacketMergeBoundary {
        &self.merge_boundary
    }

    pub(in crate::frontier_planning::testing) fn new(
        source_plan_digest: PlanDigest,
        family: PlannedWorkPacketFamily,
        ordinal: usize,
        scope_summary: String,
        merge_boundary: PacketMergeBoundary,
        basis_digest: &BundleResolvedBasisDigest,
    ) -> Self {
        let digest = PlannedWorkPacketDigest::from_parts(&[
            format!("plan:{}", source_plan_digest.as_str()),
            format!("family:{}", family.as_str()),
            format!("ordinal:{ordinal}"),
            format!("scope:{scope_summary}"),
            format!("merge:{}", merge_boundary.digest().as_str()),
            format!("basis:{}", basis_digest.as_str()),
        ]);
        Self {
            source_plan_digest,
            family,
            ordinal,
            digest,
            scope_summary,
            merge_boundary,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlannedWorkPacketSet {
    packets: Vec<PlannedWorkPacket>,
    equivalence_contract: PacketEquivalenceContract,
}

impl PlannedWorkPacketSet {
    pub(crate) fn packets(&self) -> &[PlannedWorkPacket] {
        &self.packets
    }

    pub(crate) fn equivalence_contract(&self) -> &PacketEquivalenceContract {
        &self.equivalence_contract
    }

    pub(in crate::frontier_planning::testing) fn new(
        packets: Vec<PlannedWorkPacket>,
        equivalence_contract: PacketEquivalenceContract,
    ) -> Self {
        Self {
            packets,
            equivalence_contract,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FrontierPlanningCounters {
    pub(in crate::frontier_planning::testing) frontier_planning_invocation_count: usize,
    pub(in crate::frontier_planning::testing) planned_bundle_route_count: usize,
    pub(in crate::frontier_planning::testing) predicted_breadth: usize,
    pub(in crate::frontier_planning::testing) planned_packet_merge_boundary_count: usize,
}

impl FrontierPlanningCounters {
    pub fn frontier_planning_invocation_count(&self) -> usize {
        self.frontier_planning_invocation_count
    }

    pub fn planned_bundle_route_count(&self) -> usize {
        self.planned_bundle_route_count
    }

    pub fn predicted_breadth(&self) -> usize {
        self.predicted_breadth
    }

    pub fn planned_packet_merge_boundary_count(&self) -> usize {
        self.planned_packet_merge_boundary_count
    }

    pub(in crate::frontier_planning::testing) fn single_route(
        predicted_breadth: usize,
        merge_boundary_count: usize,
    ) -> Self {
        Self {
            frontier_planning_invocation_count: 1,
            planned_bundle_route_count: 1,
            predicted_breadth,
            planned_packet_merge_boundary_count: merge_boundary_count,
        }
    }
}
