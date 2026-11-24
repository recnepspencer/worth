use crate::query::data::snapshot_pinned_plan::SnapshotPinnedQueryPlan;
use crate::query::data::{QueryParallelLegality, QueryParallelProfitability};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

use super::super::query_packetization::{
    packetized_fragment_scratch_reuse_count, packetized_query_item_count,
    packetized_query_peak_width, partition_count_for_targets, query_scope_units,
    PacketizedQueryWork,
};

pub struct PacketizedQueryMetrics {
    pub packet_count: usize,
    pub target_count: usize,
    pub(in crate::internal_port::visibility::materialization::read_records::reader) peak_width:
        usize,
    pub(in crate::internal_port::visibility::materialization::read_records::reader) scope_units:
        usize,
    pub touched_partitions: usize,
    pub(in crate::internal_port::visibility::materialization::read_records::reader) scratch_reuse_count:
        usize,
}

impl PacketizedQueryMetrics {
    pub fn from_packets(packets: &[PacketizedQueryWork]) -> Self {
        Self {
            packet_count: packets.len(),
            target_count: packetized_query_item_count(packets),
            peak_width: packetized_query_peak_width(packets),
            scope_units: query_scope_units(packets),
            touched_partitions: partition_count_for_targets(packets),
            scratch_reuse_count: packetized_fragment_scratch_reuse_count(packets),
        }
    }
}

pub fn record_query_packet_metrics(
    instrumentation: &RuntimeInstrumentation,
    plan: &SnapshotPinnedQueryPlan,
    metrics: &PacketizedQueryMetrics,
) {
    instrumentation.count_query_packet_shape(
        metrics.packet_count,
        metrics.target_count,
        metrics.peak_width,
        metrics.scope_units,
    );
    if metrics.scratch_reuse_count > 0 {
        instrumentation.count_query_fragment_scratch_reuse_by(metrics.scratch_reuse_count);
    }

    if matches!(plan.legality, QueryParallelLegality::LegalReadOnlySnapshot) {
        instrumentation.count_query_parallel_legal();
    }
    if matches!(plan.profitability, QueryParallelProfitability::Profitable) {
        instrumentation.count_query_parallel_profitable();
    }
}
