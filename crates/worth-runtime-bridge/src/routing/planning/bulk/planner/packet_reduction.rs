use crate::routing::surfaces::TruthDeltaSurfaceIdentity;
use crate::{
    input::envelope::{TruthBranchIdentity, TruthCommitIdentity},
    snapshot::TruthSnapshotIdentity,
};

use super::member_identities::{bulk_continuity_member_identity, bulk_truth_view_member_identity};

pub(super) fn plan_packet_set(
    workload_identity: BridgeWorkloadIdentity,
    planned_routes: &[BridgePlannedRoute],
) -> PlannedBridgePacketSet {
    let routing_packets = planned_routes
        .iter()
        .enumerate()
        .map(|(packet_index, route)| {
            TruthDeltaRoutingPacket::new(
                workload_identity.clone(),
                route.route_identity().clone(),
                route.source_branch().clone(),
                route.source_commit().clone(),
                route.source_snapshot().clone(),
                route
                    .lowering_summary()
                    .subscription_slice_identity()
                    .clone(),
                route.routing_summary().invalidation_target_count(),
                packet_index,
            )
        })
        .collect::<Vec<_>>();
    let mut truth_view_groups = std::collections::BTreeMap::<
        BulkTruthViewMemberIdentity,
        (
            TruthBranchIdentity,
            TruthCommitIdentity,
            TruthSnapshotIdentity,
            usize,
            usize,
        ),
    >::new();
    for route in planned_routes {
        let key = bulk_truth_view_member_identity(route);
        let entry = truth_view_groups.entry(key).or_insert_with(|| {
            (
                route.source_branch().clone(),
                route.source_commit().clone(),
                route.source_snapshot().clone(),
                0,
                0,
            )
        });
        entry.3 += 1;
        entry.4 += route.planning_summary().snapshot_read_count();
    }
    let truth_view_packets = truth_view_groups
        .into_iter()
        .enumerate()
        .map(
            |(
                packet_index,
                (
                    truth_view_member_identity,
                    (
                        source_branch,
                        source_commit,
                        source_snapshot,
                        planned_route_count,
                        snapshot_read_count,
                    ),
                ),
            )| {
                TruthViewMaterializationPacket::new(
                    workload_identity.clone(),
                    truth_view_member_identity,
                    source_branch,
                    source_commit,
                    source_snapshot,
                    planned_route_count,
                    snapshot_read_count,
                    packet_index,
                )
            },
        )
        .collect::<Vec<_>>();
    let continuity_packets = planned_routes
        .iter()
        .filter_map(|route| {
            let lineage_context = route.mapping_context().lineage_context()?;
            let prior_slice_count = route.routing_summary().invalidation_target_count();
            Some((
                route.route_identity().clone(),
                bulk_continuity_member_identity(lineage_context),
                lineage_context.authority_basis().branch_identity().clone(),
                lineage_context
                    .authority_basis()
                    .snapshot_identity()
                    .clone(),
                prior_slice_count,
            ))
        })
        .enumerate()
        .map(
            |(
                packet_index,
                (
                    route_identity,
                    continuity_member_identity,
                    branch_identity,
                    snapshot_identity,
                    prior_slice_count,
                ),
            )| {
                ContinuityRemapPacket::new(
                    workload_identity.clone(),
                    route_identity,
                    continuity_member_identity,
                    branch_identity,
                    snapshot_identity,
                    prior_slice_count,
                    packet_index,
                )
            },
        )
        .collect::<Vec<_>>();
    let widening_packets = planned_routes
        .iter()
        .flat_map(|route| {
            route
                .route_record_entries()
                .iter()
                .filter_map(|entry| {
                    entry.widening_class().map(|widening_class| {
                        (
                            route.route_identity().clone(),
                            *widening_class,
                            entry.truth_delta_surface_identity().clone(),
                        )
                    })
                })
                .collect::<Vec<_>>()
        })
        .enumerate()
        .map(
            |(packet_index, (route_identity, widening_class, bounded_scope_identity))| {
                WideningAggregationPacket::new(
                    workload_identity.clone(),
                    route_identity,
                    widening_class,
                    bounded_scope_identity,
                    packet_index,
                )
            },
        )
        .collect::<Vec<_>>();

    let mut reduction_groups = Vec::<(
        BridgeSubscriptionSliceIdentity,
        Vec<&TruthDeltaRoutingPacket>,
    )>::new();
    for packet in &routing_packets {
        let scope = packet.subscription_slice_identity().clone();
        if let Some((_, packets)) = reduction_groups
            .iter_mut()
            .find(|(existing_scope, _)| existing_scope == &scope)
        {
            packets.push(packet);
        } else {
            reduction_groups.push((scope, vec![packet]));
        }
    }
    reduction_groups.sort_by(|left, right| left.0.cmp(&right.0));
    let reduction_packets = reduction_groups
        .into_iter()
        .enumerate()
        .map(
            |(packet_index, (reduced_subscription_slice_identity, packets))| {
                let reduced_target_identity =
                    ReducedRoutingTargetIdentity::admit_bridge_owned(digest_string(
                        "reduced-routing-target",
                        &reduced_publication_packet_digest_basis(
                            &workload_identity,
                            &reduced_subscription_slice_identity,
                            &packets,
                        ),
                    ));
                InvalidationReductionPacket::new(
                    workload_identity.clone(),
                    BridgeInvalidationReductionFamily::Publication,
                    reduced_subscription_slice_identity,
                    reduced_target_identity,
                    packet_index,
                )
            },
        )
        .collect::<Vec<_>>();

    PlannedBridgePacketSet::new(
        workload_identity,
        routing_packets,
        truth_view_packets,
        continuity_packets,
        widening_packets,
        reduction_packets,
        BridgeBulkPlanningCounters::zero(),
    )
}

pub(super) fn reduce_packet_set(
    workload_identity: BridgeWorkloadIdentity,
    packet_set: &PlannedBridgePacketSet,
    counters: BridgeBulkPlanningCounters,
) -> ReducedBridgeWorkloadArtifact {
    let mut continuity_groups = std::collections::BTreeMap::<
        BulkContinuityMemberIdentity,
        (TruthBranchIdentity, TruthSnapshotIdentity, usize, usize),
    >::new();
    for packet in packet_set.continuity_packets() {
        let key = packet.continuity_member_identity().clone();
        let entry = continuity_groups.entry(key).or_insert_with(|| {
            (
                packet.typed_branch_identity().clone(),
                packet.typed_snapshot_identity().clone(),
                0,
                0,
            )
        });
        entry.2 += 1;
        entry.3 += packet.prior_slice_count();
    }
    let reduced_continuity_remaps = continuity_groups
        .into_iter()
        .map(
            |(
                continuity_member_identity,
                (branch_identity, snapshot_identity, reduced_route_count, prior_slice_count),
            )| {
                let continuity_identity =
                    ReducedContinuityIdentity::admit_bridge_owned(digest_string(
                        "reduced-continuity",
                        &format!(
                        "reduced-continuity|workload={}|continuity-member={}|branch={}|snapshot={}",
                        workload_identity.as_str(),
                        continuity_member_identity.as_str(),
                        branch_identity.as_str(),
                        snapshot_identity.as_str(),
                    ),
                    ));
                ReducedContinuityRemap::new(
                    continuity_identity,
                    continuity_member_identity,
                    branch_identity,
                    snapshot_identity,
                    reduced_route_count,
                    prior_slice_count,
                )
            },
        )
        .collect::<Vec<_>>();
    let reduced_truth_views = packet_set
        .truth_view_packets()
        .iter()
        .map(|packet| {
            let truth_view_identity = ReducedTruthViewIdentity::admit_bridge_owned(digest_string(
                "reduced-truth-view",
                &format!(
                    "reduced-truth-view|workload={}|truth-view-member={}|branch={}|commit={}|snapshot={}",
                    workload_identity.as_str(),
                    packet.truth_view_member_identity().as_str(),
                    packet.source_branch(),
                    packet.source_commit(),
                    packet.source_snapshot(),
                ),
            ));
            ReducedTruthViewMaterialization::new(
                truth_view_identity,
                packet.truth_view_member_identity().clone(),
                packet.typed_source_branch().clone(),
                packet.typed_source_commit().clone(),
                packet.typed_source_snapshot().clone(),
                packet.planned_route_count(),
                packet.snapshot_read_count(),
            )
        })
        .collect::<Vec<_>>();
    let mut widening_groups = std::collections::BTreeMap::<
        (BridgeMappingWideningClass, TruthDeltaSurfaceIdentity),
        Vec<BridgeRouteIdentity>,
    >::new();
    for packet in packet_set.widening_packets() {
        widening_groups
            .entry((
                packet.widening_class(),
                packet.bounded_truth_delta_surface_identity().clone(),
            ))
            .or_default()
            .push(packet.originating_route_identity().clone());
    }
    let reduced_widenings = widening_groups
        .into_iter()
        .map(
            |((widening_class, bounded_scope_identity), mut reduced_route_identities)| {
                reduced_route_identities.sort();
                let widening_identity = ReducedWideningIdentity::admit_bridge_owned(digest_string(
                    "reduced-widening",
                    &format!(
                        "reduced-widening|workload={}|widening-class={}|bounded-scope={}",
                        workload_identity.as_str(),
                        mapping_widening_class_basis(widening_class),
                        bounded_scope_identity.as_str(),
                    ),
                ));
                ReducedWideningAggregation::new(
                    widening_identity,
                    widening_class,
                    bounded_scope_identity,
                    reduced_route_identities,
                )
            },
        )
        .collect::<Vec<_>>();
    let reduced_publications = packet_set
        .reduction_packets()
        .iter()
        .map(|packet| {
            let routes = packet_set
                .routing_packets()
                .iter()
                .filter(|route| {
                    route.subscription_slice_identity()
                        == packet.reduced_subscription_slice_identity()
                })
                .collect::<Vec<_>>();
            let reduced_route_identities = routes
                .iter()
                .map(|route| route.route_identity().clone())
                .collect::<Vec<_>>();
            let invalidation_target_count = routes
                .iter()
                .map(|route| route.invalidation_target_count())
                .sum();
            let publication_identity = ReducedPublicationIdentity::admit_bridge_owned(digest_string(
                "reduced-publication",
                &format!(
                    "reduced-publication|workload={}|routing-target={}|subscription-slice={}|route-count={}|invalidation-target-count={}",
                    workload_identity.as_str(),
                    packet.reduced_target_identity().as_str(),
                    packet.reduced_subscription_slice_identity().as_str(),
                    reduced_route_identities.len(),
                    invalidation_target_count,
                ),
            ));
            ReducedBridgePublication::new(
                packet.reduced_target_identity().clone(),
                publication_identity,
                packet.reduced_subscription_slice_identity().clone(),
                reduced_route_identities,
                invalidation_target_count,
            )
        })
        .collect::<Vec<_>>();

    ReducedBridgeWorkloadArtifact::new(
        workload_identity,
        reduced_continuity_remaps,
        reduced_truth_views,
        reduced_widenings,
        reduced_publications,
        counters,
    )
}
use super::{admission::*, *};
