#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeInvalidationReductionFamily {
    Publication,
}

impl BridgeInvalidationReductionFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Publication => "publication",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthDeltaRoutingPacket {
    packet_identity: RoutingPacketIdentity,
    workload_identity: BridgeWorkloadIdentity,
    route_identity: BridgeRouteIdentity,
    source_branch: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_snapshot: TruthSnapshotIdentity,
    subscription_slice_identity: BridgeSubscriptionSliceIdentity,
    invalidation_target_count: usize,
    packet_index: usize,
    digest: Arc<str>,
}

impl TruthDeltaRoutingPacket {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        route_identity: BridgeRouteIdentity,
        source_branch: TruthBranchIdentity,
        source_commit: TruthCommitIdentity,
        source_snapshot: TruthSnapshotIdentity,
        subscription_slice_identity: BridgeSubscriptionSliceIdentity,
        invalidation_target_count: usize,
        packet_index: usize,
    ) -> Self {
        let basis = format!(
            "truth-delta-routing-packet|workload={}|route={}|branch={}|commit={}|snapshot={}|subscription-slice={}|invalidation-target-count={}|packet-index={}",
            workload_identity.as_str(),
            route_identity.as_str(),
            source_branch.canonical_label(),
            source_commit.canonical_label(),
            source_snapshot.canonical_label(),
            subscription_slice_identity.as_str(),
            invalidation_target_count,
            packet_index,
        );
        let packet_identity =
            RoutingPacketIdentity::admit_bridge_owned(digest_string("routing-packet", &basis));
        Self {
            packet_identity,
            workload_identity,
            route_identity,
            source_branch,
            source_commit,
            source_snapshot,
            subscription_slice_identity,
            invalidation_target_count,
            packet_index,
            digest: digest_string("truth-delta-routing-packet", &basis),
        }
    }

    pub fn packet_identity(&self) -> &RoutingPacketIdentity {
        &self.packet_identity
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn route_identity(&self) -> &BridgeRouteIdentity {
        &self.route_identity
    }

    pub fn source_commit(&self) -> String {
        self.source_commit.canonical_label()
    }

    pub fn source_branch(&self) -> String {
        self.source_branch.canonical_label()
    }

    pub(crate) fn typed_source_branch(&self) -> &TruthBranchIdentity {
        &self.source_branch
    }

    pub fn source_snapshot(&self) -> String {
        self.source_snapshot.canonical_label()
    }

    pub(crate) fn typed_source_snapshot(&self) -> &TruthSnapshotIdentity {
        &self.source_snapshot
    }

    pub fn subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.subscription_slice_identity
    }

    pub fn invalidation_target_count(&self) -> usize {
        self.invalidation_target_count
    }

    pub fn packet_index(&self) -> usize {
        self.packet_index
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidationReductionPacket {
    packet_identity: ReductionPacketIdentity,
    workload_identity: BridgeWorkloadIdentity,
    reduction_family: BridgeInvalidationReductionFamily,
    reduced_subscription_slice_identity: BridgeSubscriptionSliceIdentity,
    reduced_target_identity: ReducedRoutingTargetIdentity,
    packet_index: usize,
    digest: Arc<str>,
}

impl InvalidationReductionPacket {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        reduction_family: BridgeInvalidationReductionFamily,
        reduced_subscription_slice_identity: BridgeSubscriptionSliceIdentity,
        reduced_target_identity: ReducedRoutingTargetIdentity,
        packet_index: usize,
    ) -> Self {
        let basis = format!(
            "invalidation-reduction-packet|workload={}|reduction-family={}|reduced-target-scope={}|reduced-target-identity={}|packet-index={}",
            workload_identity.as_str(),
            reduction_family.as_str(),
            reduced_subscription_slice_identity.as_str(),
            reduced_target_identity.as_str(),
            packet_index,
        );
        let packet_identity =
            ReductionPacketIdentity::admit_bridge_owned(digest_string("reduction-packet", &basis));
        Self {
            packet_identity,
            workload_identity,
            reduction_family,
            reduced_subscription_slice_identity,
            reduced_target_identity,
            packet_index,
            digest: digest_string("invalidation-reduction-packet", &basis),
        }
    }

    pub fn packet_identity(&self) -> &ReductionPacketIdentity {
        &self.packet_identity
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn reduction_family(&self) -> BridgeInvalidationReductionFamily {
        self.reduction_family
    }

    pub fn reduction_family_label(&self) -> &'static str {
        self.reduction_family.as_str()
    }

    pub fn reduced_subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.reduced_subscription_slice_identity
    }

    pub fn reduced_target_identity(&self) -> &ReducedRoutingTargetIdentity {
        &self.reduced_target_identity
    }

    pub fn packet_index(&self) -> usize {
        self.packet_index
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedBridgePacketSet {
    workload_identity: BridgeWorkloadIdentity,
    routing_packets: Arc<[TruthDeltaRoutingPacket]>,
    truth_view_packets: Arc<[TruthViewMaterializationPacket]>,
    continuity_packets: Arc<[ContinuityRemapPacket]>,
    widening_packets: Arc<[WideningAggregationPacket]>,
    reduction_packets: Arc<[InvalidationReductionPacket]>,
    counters: BridgeBulkPlanningCounters,
    digest: Arc<str>,
}

impl PlannedBridgePacketSet {
    pub(crate) fn new(
        workload_identity: BridgeWorkloadIdentity,
        routing_packets: Vec<TruthDeltaRoutingPacket>,
        truth_view_packets: Vec<TruthViewMaterializationPacket>,
        continuity_packets: Vec<ContinuityRemapPacket>,
        widening_packets: Vec<WideningAggregationPacket>,
        reduction_packets: Vec<InvalidationReductionPacket>,
        counters: BridgeBulkPlanningCounters,
    ) -> Self {
        let mut basis = format!(
            "planned-bridge-packet-set|workload={}|routing-count={}|truth-view-count={}|continuity-count={}|widening-count={}|reduction-count={}",
            workload_identity.as_str(),
            routing_packets.len(),
            truth_view_packets.len(),
            continuity_packets.len(),
            widening_packets.len(),
            reduction_packets.len(),
        );
        for packet in &routing_packets {
            basis.push_str("|routing=");
            basis.push_str(packet.digest());
        }
        for packet in &truth_view_packets {
            basis.push_str("|truth-view=");
            basis.push_str(packet.digest());
        }
        for packet in &continuity_packets {
            basis.push_str("|continuity=");
            basis.push_str(packet.digest());
        }
        for packet in &widening_packets {
            basis.push_str("|widening=");
            basis.push_str(packet.digest());
        }
        for packet in &reduction_packets {
            basis.push_str("|reduction=");
            basis.push_str(packet.digest());
        }
        Self {
            workload_identity,
            routing_packets: routing_packets.into(),
            truth_view_packets: truth_view_packets.into(),
            continuity_packets: continuity_packets.into(),
            widening_packets: widening_packets.into(),
            reduction_packets: reduction_packets.into(),
            counters,
            digest: digest_string("planned-bridge-packet-set", &basis),
        }
    }

    pub fn workload_identity(&self) -> &BridgeWorkloadIdentity {
        &self.workload_identity
    }

    pub fn routing_packets(&self) -> &[TruthDeltaRoutingPacket] {
        &self.routing_packets
    }

    pub fn truth_view_packets(&self) -> &[TruthViewMaterializationPacket] {
        &self.truth_view_packets
    }

    pub fn continuity_packets(&self) -> &[ContinuityRemapPacket] {
        &self.continuity_packets
    }

    pub fn widening_packets(&self) -> &[WideningAggregationPacket] {
        &self.widening_packets
    }

    pub fn reduction_packets(&self) -> &[InvalidationReductionPacket] {
        &self.reduction_packets
    }

    pub fn counters(&self) -> &BridgeBulkPlanningCounters {
        &self.counters
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub(crate) fn with_counters(&self, counters: BridgeBulkPlanningCounters) -> Self {
        Self {
            workload_identity: self.workload_identity.clone(),
            routing_packets: Arc::clone(&self.routing_packets),
            truth_view_packets: Arc::clone(&self.truth_view_packets),
            continuity_packets: Arc::clone(&self.continuity_packets),
            widening_packets: Arc::clone(&self.widening_packets),
            reduction_packets: Arc::clone(&self.reduction_packets),
            counters,
            digest: Arc::clone(&self.digest),
        }
    }
}

use super::*;
