use crate::identity::PlanDigest;

use super::{
    BundleResolvedBasisDigest, FrontierAwarePlan, FrontierBreadthPrediction, FrontierPlanFamily,
    FrontierPostureDigest, FrontierPredictionDriftOutcome, FrontierSurfaceDigest,
    PacketMergeContract, PlannedWorkPacketSet, SerialFallbackEvidence, SerialFallbackReason,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierPlanningReport {
    posture_digest: FrontierPostureDigest,
    family: FrontierPlanFamily,
    source_plan_digest: PlanDigest,
    bundle_basis_digest: BundleResolvedBasisDigest,
    predicted_breadth: FrontierBreadthPrediction,
    packet_merge_contract: PacketMergeContract,
    packet_count: usize,
    packet_merge_boundary_count: usize,
}

impl FrontierPlanningReport {
    pub fn posture_digest(&self) -> &FrontierPostureDigest {
        &self.posture_digest
    }

    pub fn source_plan_digest(&self) -> &PlanDigest {
        &self.source_plan_digest
    }

    pub fn predicted_breadth(&self) -> &FrontierBreadthPrediction {
        &self.predicted_breadth
    }

    pub(crate) fn packet_merge_contract(&self) -> &PacketMergeContract {
        &self.packet_merge_contract
    }

    pub(in crate::frontier_planning::testing) fn new(
        family: FrontierPlanFamily,
        source_plan_digest: PlanDigest,
        bundle_basis_digest: BundleResolvedBasisDigest,
        predicted_breadth: FrontierBreadthPrediction,
        packet_set: &PlannedWorkPacketSet,
    ) -> Self {
        let mut parts = vec![
            format!("family:{}", family.as_str()),
            format!("plan:{}", source_plan_digest.as_str()),
            format!("basis:{}", bundle_basis_digest.as_str()),
            format!("predicted_breadth:{}", predicted_breadth.value()),
            format!(
                "packet_equivalence:{}",
                packet_set.equivalence_contract().as_str()
            ),
        ];
        for packet in packet_set.packets() {
            parts.push(format!("packet:{}", packet.digest().as_str()));
            parts.push(format!(
                "merge:{}",
                packet.merge_boundary().digest().as_str()
            ));
        }

        Self {
            posture_digest: FrontierPostureDigest::from_parts(&parts),
            family,
            source_plan_digest,
            bundle_basis_digest,
            predicted_breadth,
            packet_merge_contract: packet_set.packets()[0].merge_boundary().contract().clone(),
            packet_count: packet_set.packets().len(),
            packet_merge_boundary_count: packet_set.packets().len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierRouteReport {
    posture_digest: FrontierPostureDigest,
    source_plan_digest: PlanDigest,
    route_surface_digest: FrontierSurfaceDigest,
    predicted_breadth: FrontierBreadthPrediction,
    drift_outcome: FrontierPredictionDriftOutcome,
    serial_fallback_reason: SerialFallbackReason,
}

impl FrontierRouteReport {
    pub fn posture_digest(&self) -> &FrontierPostureDigest {
        &self.posture_digest
    }

    pub fn source_plan_digest(&self) -> &PlanDigest {
        &self.source_plan_digest
    }

    pub fn route_surface_digest(&self) -> &FrontierSurfaceDigest {
        &self.route_surface_digest
    }

    pub fn predicted_breadth(&self) -> &FrontierBreadthPrediction {
        &self.predicted_breadth
    }

    pub fn drift_outcome(&self) -> &FrontierPredictionDriftOutcome {
        &self.drift_outcome
    }

    pub fn serial_fallback_reason(&self) -> &SerialFallbackReason {
        &self.serial_fallback_reason
    }

    pub(in crate::frontier_planning::testing) fn from_serial_route(
        posture_digest: FrontierPostureDigest,
        frontier_plan: &FrontierAwarePlan,
        evidence: &SerialFallbackEvidence,
    ) -> Self {
        Self {
            posture_digest,
            source_plan_digest: frontier_plan.source_plan_digest().clone(),
            route_surface_digest: evidence.surface_digest().clone(),
            predicted_breadth: frontier_plan.predicted_breadth().clone(),
            drift_outcome: evidence.drift_outcome().clone(),
            serial_fallback_reason: evidence.reason().clone(),
        }
    }
}
