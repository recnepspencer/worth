use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionReport;

use super::super::query_packetization::PacketizedQueryWork;
use super::super::{SnapshotPinnedQueryPlan, VisibilityReadContext};
use super::leased_explicit::execute_leased_explicit_fragment;
use super::leased_map::{execute_leased_query_packets, QueryReadExecutionStop};
use crate::storage::overlay::PartitionAccess;

pub(in crate::visibility::materialization::read_records::reader) fn execute_explicit_query_fragments_from_exact_basis(
    reader: &VisibilityReadContext<'_>,
    plan: &SnapshotPinnedQueryPlan,
    packets: Vec<PacketizedQueryWork>,
    basis: &crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<
    (
        Vec<crate::query::data::QueryWorkerFragment>,
        ExecutionReport,
    ),
    QueryReadExecutionStop,
> {
    let state_access: &(dyn PartitionAccess + Sync) = basis.root().as_ref();
    let registry = basis.root().schema_authority().registry();
    let version_id = basis.version_id();
    execute_leased_query_packets(
        packets,
        execution,
        lease,
        |packet, ordinal, ceiling, context| {
            execute_leased_explicit_fragment(
                reader,
                basis,
                state_access,
                registry,
                version_id,
                &plan.packet,
                packet,
                ordinal as u64,
                ceiling,
                context,
            )
        },
    )
}
