//! One checked aspect-version enumeration for either request backing.
use super::{query_execution::QueryReadExecutionStop, RelationalRuntime};
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;
use worth_execution::{
    ChargedBytes, ExecutionRequest, ExecutionScan, MapKernelFailure, ScanOutcome,
};
use worth_foundational::{facade::AspectKey, PartitionIdentity};

struct ReadVersions {
    versions: Vec<(AspectKey, u64)>,
    owned_bytes: u64,
}

impl ChargedBytes for ReadVersions {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// One work unit per symbolic version visited; relation ordering additionally
/// admits n * ceil(log2(n)) comparison units before its existing sort.
pub(super) fn read_versions(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    symbolic: &BTreeMap<Symbol, u64>,
    sort: bool,
) -> Result<Vec<(AspectKey, u64)>, QueryReadExecutionStop> {
    super::query_preparation::check_query_preparation_request(execution)?;
    execution
        .in_scope(|lease| {
            let identity = PartitionIdentity::new(1);
            let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
                .map_err(QueryReadExecutionStop::PreparationAdmission)?;
            let ceiling = execution.memory_limit() / 4;
            let outcome = scan.run(lease, (), 0, ceiling, 0, ceiling, |_, _, context| {
                context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                let mut versions = Vec::new();
                let mut payload_bytes = 0_u64;
                runtime.services.symbols.with_read(|symbols| {
                    for (symbol, version) in symbolic {
                        context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
                        let Some(text) = symbols.resolve(*symbol) else {
                            continue;
                        };
                        let required = payload_bytes
                            .checked_add(text.len() as u64)
                            .and_then(|bytes| {
                                bytes.checked_add(
                                    (versions.capacity().max(versions.len().saturating_add(1))
                                        as u64)
                                        .saturating_mul(
                                            std::mem::size_of::<(AspectKey, u64)>() as u64
                                        ),
                                )
                            })
                            .and_then(|bytes| {
                                bytes.checked_add(std::mem::size_of::<ReadVersions>() as u64)
                            })
                            .filter(|bytes| *bytes <= ceiling)
                            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
                        let Some(key) = AspectKey::new(text) else {
                            continue;
                        };
                        versions
                            .try_reserve_exact(1)
                            .map_err(|_| MapKernelFailure::ResultCapacityExceeded)?;
                        let actual =
                            payload_bytes
                                .saturating_add(text.len() as u64)
                                .saturating_add(
                                    (versions.capacity() as u64).saturating_mul(
                                        std::mem::size_of::<(AspectKey, u64)>() as u64,
                                    ),
                                )
                                .saturating_add(std::mem::size_of::<ReadVersions>() as u64);
                        if actual > ceiling {
                            return Err(MapKernelFailure::ResultCapacityExceeded);
                        }
                        debug_assert!(actual >= required);
                        payload_bytes = payload_bytes.saturating_add(text.len() as u64);
                        versions.push((key, *version));
                    }
                    Ok::<(), MapKernelFailure<()>>(())
                })?;
                if sort {
                    let n = versions.len();
                    let work = if n <= 1 {
                        0
                    } else {
                        (n as u64).saturating_mul(u64::from(usize::BITS - (n - 1).leading_zeros()))
                    };
                    context.checkpoint(work).map_err(MapKernelFailure::Stop)?;
                    versions.sort();
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                }
                let owned_bytes = payload_bytes.saturating_add(
                    (versions.capacity() as u64)
                        .saturating_mul(std::mem::size_of::<(AspectKey, u64)>() as u64),
                );
                Ok((
                    (),
                    ReadVersions {
                        versions,
                        owned_bytes,
                    },
                ))
            });
            match outcome {
                ScanOutcome::Complete { mut prefixes, .. } => {
                    Ok(prefixes.pop().expect("one aspect-version read").versions)
                }
                ScanOutcome::Stopped { reason, report, .. } => {
                    Err(QueryReadExecutionStop::PreparationStopped { reason, report })
                }
            }
        })
        .map_err(QueryReadExecutionStop::Request)?
}
