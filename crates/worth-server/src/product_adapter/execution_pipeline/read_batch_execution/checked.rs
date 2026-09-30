use std::{cell::RefCell, mem::size_of, sync::Arc};

use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, ExecutionScan, MapDenial, MapKernelFailure,
    MapKernelStop, MapOutcome, MapPartition, ScanDenial, ScanOutcome,
};
use worth_foundational::PartitionIdentity;

use super::*;

const NESTED_READ_BUDGET_DIVISOR: u64 = 32;
const BATCH_SCRATCH_BUDGET_DIVISOR: u64 = 8;

struct BatchInput {
    inputs: RefCell<Option<Vec<WorthServerProductOperationInput>>>,
    owned_bytes: u64,
}

impl ChargedBytes for BatchInput {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

struct BatchOutput {
    batch: WorthServerExecutedProductReadBatch,
    owned_bytes: u64,
}

impl ChargedBytes for BatchOutput {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

enum DomainStop {
    Preparation(WorthServerProductOperationSurfaceDenial),
    Admission(MapDenial),
    Packet {
        boundary: Option<PartitionIdentity>,
        reason: worth_execution::MapStop<()>,
    },
    Finalization {
        ordinal: usize,
        denial: WorthServerProductOperationSurfaceDenial,
    },
}

pub(super) fn execute(
    operation_registry: &WorthServerOperationRegistry,
    adapter_registry: &WorthServerProductAdapterRegistry,
    query_handoff_config: &WorthServerQueryHandoffConfig,
    admission: &WorthServerAdmission,
    inputs: Vec<WorthServerProductOperationInput>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<WorthServerExecutedProductReadBatch, WorthServerProductReadBatchStop> {
    if lease.is_cancelled() {
        return Err(WorthServerProductReadBatchStop::Preflight(
            MapKernelStop::Cancelled,
        ));
    }
    if lease.deadline_elapsed() {
        return Err(WorthServerProductReadBatchStop::Preflight(
            MapKernelStop::DeadlineElapsed,
        ));
    }
    let count = inputs.len();
    if (count as u64) > lease.policy().budget().work_ceiling() {
        return Err(WorthServerProductReadBatchStop::Preflight(
            MapKernelStop::WorkCeiling,
        ));
    }
    let descriptor_bytes = (count as u64).saturating_mul(
        (size_of::<Arc<WorthServerPreparedProductReadSlot>>()
            + size_of::<MapPartition<PreparedReadPacket, u64>>()
            + 2 * size_of::<PartitionIdentity>()) as u64,
    );
    if descriptor_bytes > lease.policy().budget().charged_memory_bytes() / 4 {
        return Err(WorthServerProductReadBatchStop::PacketAdmission(
            MapDenial::MemoryOverflow,
        ));
    }
    let mut input_bytes = (inputs.capacity() as u64)
        .saturating_mul(size_of::<WorthServerProductOperationInput>() as u64);
    for input in &inputs {
        if lease.is_cancelled() {
            return Err(WorthServerProductReadBatchStop::Preflight(
                MapKernelStop::Cancelled,
            ));
        }
        if lease.deadline_elapsed() {
            return Err(WorthServerProductReadBatchStop::Preflight(
                MapKernelStop::DeadlineElapsed,
            ));
        }
        input_bytes = input_bytes.saturating_add(input.owned_allocation_capacity_bytes());
    }
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(
        vec![identity],
        vec![(
            identity,
            BatchInput {
                inputs: RefCell::new(Some(inputs)),
                owned_bytes: input_bytes,
            },
        )],
    )
    .map_err(|denial| {
        WorthServerProductReadBatchStop::PacketAdmission(match denial {
            ScanDenial::IdentitiesNotCanonical => MapDenial::ExpectedIdentitiesNotCanonical,
            ScanDenial::CoverageMismatch => MapDenial::CoverageMismatch,
            ScanDenial::MemoryOverflow => MapDenial::MemoryOverflow,
        })
    })?;
    let memory = lease.policy().budget().charged_memory_bytes();
    let ceiling = (memory / BATCH_SCRATCH_BUDGET_DIVISOR)
        .max(16 * 1024)
        .min(memory / 4);
    let mut domain_stop = None;
    let outcome = scan.run(
        Some(lease),
        (),
        0,
        ceiling,
        0,
        ceiling,
        |_, input, context| {
            context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
            let inputs = input
                .inputs
                .borrow_mut()
                .take()
                .expect("one batch scan step");
            let mut claimed = descriptor_bytes;
            let mut prepared = Vec::new();
            prepared
                .try_reserve_exact(count)
                .map_err(|_| MapKernelFailure::<()>::ResultCapacityExceeded)?;
            for input in inputs {
                let input_bytes = input.owned_allocation_capacity_bytes();
                context
                    .checkpoint(1_u64.saturating_add(input_bytes / 1024))
                    .map_err(MapKernelFailure::Stop)?;
                claim(
                    &mut claimed,
                    ceiling,
                    input_bytes.saturating_mul(4).saturating_add(1024),
                )?;
                if let Some((_, declaration)) = adapter_registry.resolve(input.operation_name()) {
                    claim(
                        &mut claimed,
                        ceiling,
                        declaration
                            .owned_allocation_capacity_bytes()
                            .saturating_mul(4)
                            .saturating_add(
                                admission
                                    .owned_allocation_capacity_bytes()
                                    .saturating_mul(4),
                            ),
                    )?;
                }
                let slot = match prepare_shared_read_slot(
                    operation_registry,
                    adapter_registry,
                    query_handoff_config,
                    admission,
                    input,
                ) {
                    Ok(slot) => slot,
                    Err(denial) => {
                        domain_stop = Some(DomainStop::Preparation(denial));
                        return Err(MapKernelFailure::<()>::Domain(()));
                    }
                };
                context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                claim(
                    &mut claimed,
                    ceiling,
                    slot.owned_allocation_capacity_bytes(),
                )?;
                prepared.push(Arc::new(slot));
            }
            let per_result_ceiling = lease.policy().budget().charged_memory_bytes()
                / (NESTED_READ_BUDGET_DIVISOR
                    .saturating_mul(count as u64)
                    .max(1));
            let mut identities = Vec::new();
            let mut partitions = Vec::new();
            identities
                .try_reserve_exact(count)
                .map_err(|_| MapKernelFailure::<()>::ResultCapacityExceeded)?;
            partitions
                .try_reserve_exact(count)
                .map_err(|_| MapKernelFailure::<()>::ResultCapacityExceeded)?;
            for (ordinal, slot) in prepared.iter().enumerate() {
                context.checkpoint(1).map_err(MapKernelFailure::Stop)?;
                let identity = PartitionIdentity::new(ordinal as u64 + 1);
                identities.push(identity);
                partitions.push(MapPartition {
                    identity,
                    value: PreparedReadPacket {
                        slot: Arc::clone(slot),
                    },
                    read_keys: Vec::<u64>::new(),
                    write_keys: Vec::new(),
                    kernel_scratch_bytes: per_result_ceiling,
                    max_result_bytes: per_result_ceiling,
                });
            }
            let map = match ExecutionMap::try_from_declared_partitions(identities, partitions) {
                Ok(map) => map,
                Err(denial) => {
                    domain_stop = Some(DomainStop::Admission(denial));
                    return Err(MapKernelFailure::<()>::Domain(()));
                }
            };
            let adapter_results = match map.run(Some(lease), |packet, kernel| {
                kernel.checkpoint(1)?;
                let outcome = packet.slot.adapter.execute_with_lease(
                    &packet.slot.scheduled,
                    lease,
                    kernel,
                )?;
                kernel.checkpoint(0)?;
                Ok::<_, MapKernelFailure<()>>(AdapterReadResult { outcome })
            }) {
                MapOutcome::Complete { values, .. } => values,
                MapOutcome::Stopped {
                    boundary, reason, ..
                } => {
                    domain_stop = Some(DomainStop::Packet { boundary, reason });
                    return Err(MapKernelFailure::<()>::Domain(()));
                }
            };
            context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
            let mut operations = Vec::new();
            operations
                .try_reserve_exact(count)
                .map_err(|_| MapKernelFailure::<()>::ResultCapacityExceeded)?;
            let mut counters = WorthServerOperationSchedulerCounters::default();
            counters.set_planned_batch_width(count);
            counters.increment_admitted_read_slot_count_by(count);
            counters.increment_queued_read_slot_count_by(count);
            counters.increment_completed_read_slot_count_by(count);
            claim(&mut claimed, ceiling, 256)?;
            let mut digest =
                format!("worth-server-product-read-batch-v1|counters={counters:?}|operations=");
            for (ordinal, (slot, result)) in prepared.iter().zip(adapter_results).enumerate() {
                let result_bytes = result.additional_charged_bytes();
                context
                    .checkpoint(1_u64.saturating_add(result_bytes / 1024))
                    .map_err(MapKernelFailure::Stop)?;
                claim(
                    &mut claimed,
                    ceiling,
                    slot.owned_allocation_capacity_bytes()
                        .saturating_add(result_bytes.saturating_mul(3))
                        .saturating_add(1024),
                )?;
                let operation = match finalize_shared_read_slot(ordinal, slot, result) {
                    Ok(operation) => operation,
                    Err(denial) => {
                        if let WorthServerProductReadBatchStop::Finalization { ordinal, denial } =
                            denial
                        {
                            domain_stop = Some(DomainStop::Finalization { ordinal, denial });
                            return Err(MapKernelFailure::<()>::Domain(()));
                        }
                        unreachable!("finalizer returns only finalization denial");
                    }
                };
                if ordinal != 0 {
                    digest.push('|');
                }
                digest.push_str(operation.envelope().canonical_digest());
                operations.push(operation);
                context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
            }
            claim(&mut claimed, ceiling, digest.capacity() as u64)?;
            let batch = WorthServerExecutedProductReadBatch::from_checked_parts(
                operations, counters, digest,
            );
            Ok((
                (),
                BatchOutput {
                    batch,
                    owned_bytes: claimed,
                },
            ))
        },
    );
    match outcome {
        ScanOutcome::Complete {
            mut prefixes,
            report,
            ..
        } => Ok(prefixes
            .pop()
            .expect("one batch output")
            .batch
            .with_execution_report(report)),
        ScanOutcome::Stopped {
            boundary,
            reason,
            report,
            ..
        } => match domain_stop {
            Some(DomainStop::Preparation(denial)) => {
                Err(WorthServerProductReadBatchStop::Preparation(denial))
            }
            Some(DomainStop::Admission(denial)) => {
                Err(WorthServerProductReadBatchStop::PacketAdmission(denial))
            }
            Some(DomainStop::Packet { boundary, reason }) => {
                Err(WorthServerProductReadBatchStop::PacketStopped {
                    boundary,
                    reason,
                    report,
                })
            }
            Some(DomainStop::Finalization { ordinal, denial }) => {
                Err(WorthServerProductReadBatchStop::Finalization { ordinal, denial })
            }
            None => Err(WorthServerProductReadBatchStop::PacketStopped {
                boundary,
                reason,
                report,
            }),
        },
    }
}

fn claim(claimed: &mut u64, ceiling: u64, bytes: u64) -> Result<(), MapKernelFailure<()>> {
    *claimed = claimed
        .checked_add(bytes)
        .filter(|total| *total <= ceiling)
        .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
    Ok(())
}
