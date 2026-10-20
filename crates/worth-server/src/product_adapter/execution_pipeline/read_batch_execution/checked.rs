use std::{
    cell::RefCell,
    mem::size_of,
    sync::{Arc, Mutex},
};

use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionMemoryReservation, ExecutionResourceLease, ExecutionScan,
    LeaseDenial, MapDenial, MapKernelFailure, MapKernelStop, MapOutcome, MapPartition, ScanDenial,
    ScanOutcome,
};
use worth_foundational::PartitionIdentity;

use super::*;

struct BatchInput {
    inputs: RefCell<Option<Vec<WorthServerProductOperationInput>>>,
    owned_bytes: u64,
}

impl ChargedBytes for BatchInput {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

mod result_custody;
use result_custody::{execute_packet, result_memory_stop, ReadResultSlot, ReservedReadResult};

struct OwnedReadPacket {
    packet: PreparedReadPacket,
    ordinal: usize,
}

impl ChargedBytes for OwnedReadPacket {
    fn additional_charged_bytes(&self) -> u64 {
        self.packet.additional_charged_bytes()
    }
}

enum DomainStop {
    Preparation(WorthServerProductOperationSurfaceDenial),
    Admission(MapDenial),
    Memory(LeaseDenial),
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
    if lease
        .policy()
        .budget()
        .work_ceiling()
        .is_some_and(|limit| count as u64 > limit)
    {
        return Err(WorthServerProductReadBatchStop::Preflight(
            MapKernelStop::WorkCeiling,
        ));
    }
    let descriptor_bytes = (count as u64).saturating_mul(
        (size_of::<Arc<WorthServerPreparedProductReadSlot>>()
            + size_of::<MapPartition<OwnedReadPacket, u64>>()
            + 2 * size_of::<PartitionIdentity>()) as u64,
    );
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
    let mut domain_stop = None;
    let mut completed = None;
    let outcome = scan.run(Some(lease), (), 0, 0, 0, 0, |_, input, context| {
        context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
        let inputs = input
            .inputs
            .borrow_mut()
            .take()
            .expect("one batch scan step");
        let mut reservation = ExecutionMemoryReservation::reserve_in_scope(Some(lease), 0)
            .map_err(|denial| {
                domain_stop = Some(DomainStop::Memory(denial));
                MapKernelFailure::<()>::Domain(())
            })?;
        claim(&mut reservation, &mut domain_stop, descriptor_bytes)?;
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
                &mut reservation,
                &mut domain_stop,
                input_bytes.saturating_mul(4).saturating_add(1024),
            )?;
            if let Some((_, declaration)) = adapter_registry.resolve(input.operation_name()) {
                claim(
                    &mut reservation,
                    &mut domain_stop,
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
                &mut reservation,
                &mut domain_stop,
                slot.owned_allocation_capacity_bytes().saturating_add(
                    (size_of::<WorthServerPreparedProductReadSlot>() + 2 * size_of::<usize>())
                        as u64,
                ),
            )?;
            prepared.push(Arc::new(slot));
        }
        claim(
            &mut reservation,
            &mut domain_stop,
            (count as u64)
                .checked_mul(size_of::<ReadResultSlot>() as u64)
                .ok_or(MapKernelFailure::<()>::ResultCapacityExceeded)?,
        )?;
        let results: Vec<ReadResultSlot> = (0..count).map(|_| Mutex::new(None)).collect();
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
                value: OwnedReadPacket {
                    packet: PreparedReadPacket {
                        slot: Arc::clone(slot),
                    },
                    ordinal,
                },
                read_keys: Vec::<u64>::new(),
                write_keys: Vec::new(),
                // The adapter owns nested operation reservations. This map
                // returns unit; the request owns measured results below.
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            });
        }
        let map = match ExecutionMap::try_from_declared_partitions(identities, partitions) {
            Ok(map) => map,
            Err(denial) => {
                domain_stop = Some(DomainStop::Admission(denial));
                return Err(MapKernelFailure::<()>::Domain(()));
            }
        };
        match map.run(Some(lease), |packet, kernel| {
            kernel.checkpoint(1)?;
            let reserved = match execute_packet(&packet.packet, lease) {
                Ok(result) => Ok(result),
                Err(worth_execution::ScopeStop::Admission(denial)) => Err(denial),
                Err(worth_execution::ScopeStop::Failure(cause)) => return Err(cause),
            };
            let denied = reserved.is_err();
            *results[packet.ordinal]
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(reserved);
            if denied {
                return Err(MapKernelFailure::<()>::Domain(()));
            }
            Ok::<_, MapKernelFailure<()>>(())
        }) {
            MapOutcome::Complete { .. } => (),
            MapOutcome::Stopped {
                boundary, reason, ..
            } => {
                let reason = result_memory_stop(reason, &results);
                domain_stop = Some(DomainStop::Packet { boundary, reason });
                return Err(MapKernelFailure::<()>::Domain(()));
            }
        };
        context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
        claim(
            &mut reservation,
            &mut domain_stop,
            (count as u64)
                .checked_mul(size_of::<WorthServerCompletedProductOperation>() as u64)
                .ok_or(MapKernelFailure::<()>::ResultCapacityExceeded)?,
        )?;
        let mut operations = Vec::new();
        operations
            .try_reserve_exact(count)
            .map_err(|_| MapKernelFailure::<()>::ResultCapacityExceeded)?;
        let mut counters = WorthServerOperationSchedulerCounters::default();
        counters.set_planned_batch_width(count);
        counters.increment_admitted_read_slot_count_by(count);
        counters.increment_queued_read_slot_count_by(count);
        counters.increment_completed_read_slot_count_by(count);
        claim(&mut reservation, &mut domain_stop, 256)?;
        let mut digest =
            format!("worth-server-product-read-batch-v1|counters={counters:?}|operations=");
        for (ordinal, (slot, result)) in prepared.iter().zip(results).enumerate() {
            let reserved = result
                .into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .expect("completed read packet has a reserved result")
                .expect("completed read packet has no memory denial");
            let ReservedReadResult {
                result,
                _memory: result_memory,
            } = reserved;
            let result_bytes = result.additional_charged_bytes();
            context
                .checkpoint(1_u64.saturating_add(result_bytes / 1024))
                .map_err(MapKernelFailure::Stop)?;
            claim(
                &mut reservation,
                &mut domain_stop,
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
            drop(result_memory);
            context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
        }
        claim(&mut reservation, &mut domain_stop, digest.capacity() as u64)?;
        let batch =
            WorthServerExecutedProductReadBatch::from_checked_parts(operations, counters, digest);
        completed = Some((batch, reservation));
        Ok(((), ()))
    });
    match outcome {
        ScanOutcome::Complete { report, .. } => {
            let (batch, _memory) = completed.expect("one completed batch owner");
            Ok(batch.with_execution_report(report))
        }
        ScanOutcome::Stopped {
            boundary,
            reason,
            report,
            ..
        } => match domain_stop {
            Some(DomainStop::Preparation(denial)) => {
                Err(WorthServerProductReadBatchStop::Preparation(denial))
            }
            Some(DomainStop::Memory(denial)) => {
                Err(WorthServerProductReadBatchStop::PacketStopped {
                    boundary,
                    reason: worth_execution::MapStop::Admission(denial),
                    report,
                })
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

fn claim(
    reservation: &mut ExecutionMemoryReservation,
    domain_stop: &mut Option<DomainStop>,
    bytes: u64,
) -> Result<(), MapKernelFailure<()>> {
    let next = reservation
        .bytes()
        .checked_add(bytes)
        .ok_or(LeaseDenial::ChargedBytesOverflow)
        .and_then(|next| {
            reservation
                .resize(next)
                .map_err(LeaseDenial::MemoryExhausted)
        });
    next.map_err(|denial| {
        *domain_stop = Some(DomainStop::Memory(denial));
        MapKernelFailure::Domain(())
    })
}
