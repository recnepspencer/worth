use worth_execution::ExecutionResourceLease;
use worth_foundational::ExecutionReport;

use super::super::query_fragment_work::execute_explicit_query_fragment_from_exact_basis;
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
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<
    Option<(
        Vec<crate::query::data::QueryWorkerFragment>,
        Option<ExecutionReport>,
    )>,
    QueryReadExecutionStop,
> {
    let state_access: &(dyn PartitionAccess + Sync) = basis.root().as_ref();
    let registry = basis.root().schema_authority().registry();
    let version_id = basis.version_id();
    match lease {
        None => {
            reader
                .runtime()
                .performance_access()
                .count_query_serial_strategy();
            Ok(packets
                .iter()
                .enumerate()
                .map(|(ordinal, packet)| {
                    execute_explicit_query_fragment_from_exact_basis(
                        reader,
                        basis,
                        state_access,
                        registry,
                        version_id,
                        &plan.packet,
                        packet,
                        ordinal as u64,
                    )
                })
                .collect::<Option<Vec<_>>>()
                .map(|fragments| (fragments, None)))
        }
        Some(lease) => {
            reader
                .runtime()
                .performance_access()
                .count_query_staged_parallel_strategy();
            let (fragments, report) = execute_leased_query_packets(
                packets,
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
            )?;
            Ok(Some((fragments, Some(report))))
        }
    }
}
