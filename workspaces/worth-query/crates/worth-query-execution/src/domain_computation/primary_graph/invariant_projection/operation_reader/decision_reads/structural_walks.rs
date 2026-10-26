//! Issued values are consumed only by their original admitted projection.
use super::*;
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity as Admission;
use crate::domain_computation::primary_graph::invariant_projection::{
    StructuralWalkKey, StructuralWalkObservation,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts as Contracts;
use worth_relational::facade::{
    snapshots::SnapshotHandle, transactions::RelationalExecutionDenialCause as Cause,
};

struct Issued {
    body: Arc<StructuralWalkObservation>,
    admission: Admission,
    contracts: Arc<Contracts>,
    cookie: u64,
}
pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) enum WalkRecordDenial
{
    Execution(Cause),
    Conflict,
}
impl From<Cause> for WalkRecordDenial {
    fn from(cause: Cause) -> Self {
        Self::Execution(cause)
    }
}
#[derive(Default)]
pub(super) struct IssuedWalkValues {
    values: BTreeMap<StructuralWalkKey, Issued>,
    bytes: u64,
    maximum_roots: usize,
    memory: Option<ExecutionMemoryReservation>,
}
impl IssuedWalkValues {
    pub(super) fn take_memory(&mut self) -> Option<ExecutionMemoryReservation> {
        self.memory.take()
    }
    pub(super) fn get(
        &self,
        key: &StructuralWalkKey,
        admission: Admission,
        contracts: &Arc<Contracts>,
        snapshot: &SnapshotHandle,
    ) -> Option<&Arc<StructuralWalkObservation>> {
        let issued = self.values.get(key)?;
        (issued.admission == admission
            && Arc::ptr_eq(&issued.contracts, contracts)
            && issued.body.issued_at(snapshot, issued.cookie))
        .then_some(&issued.body)
    }
}
fn insertion_bytes<T>(len: usize) -> Option<u64> {
    let after = len.checked_add(1)?;
    let levels = (usize::BITS - after.leading_zeros()) as u64 + 1;
    levels.checked_mul(
        11_u64
            .checked_mul(size_of::<T>() as u64)?
            .checked_add(14 * size_of::<usize>() as u64)?,
    )
}
impl DecisionReads {
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn fail_walk(
        &mut self,
    ) {
        self.failed_walk = true;
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn record_walk(
        &mut self,
        body: Arc<StructuralWalkObservation>,
        admission: Admission,
        contracts: Arc<Contracts>,
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), WalkRecordDenial> {
        // Bound the new variable-width key comparisons before the map descent.
        let levels =
            (usize::BITS - self.walks.values.len().saturating_add(1).leading_zeros()) as usize + 1;
        let units = self
            .walks
            .maximum_roots
            .max(body.key().roots().len())
            .checked_add(body.key().relation().len())
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(levels))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        (if context.has_accounting() {
            context.checkpoint(units)
        } else {
            context.poll()
        })
        .map_err(|stop| Cause::from(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let prior = self.walks.values.get(body.key());
        if let Some(prior) = prior {
            if prior.admission != admission
                || !Arc::ptr_eq(&prior.contracts, &contracts)
                || !body.issued_at(prior.body.source(), prior.cookie)
            {
                return Err(WalkRecordDenial::Conflict);
            }
            if body.native().visits().len() != prior.body.native().visits().len()
                || body.native().relations().len() != prior.body.native().relations().len()
            {
                return Err(WalkRecordDenial::Conflict);
            }
            for (a, b) in body
                .native()
                .visits()
                .iter()
                .zip(prior.body.native().visits())
            {
                StructuralWalkObservation::checkpoint(context)?;
                if a != b {
                    return Err(WalkRecordDenial::Conflict);
                }
            }
            for (a, b) in body
                .native()
                .relations()
                .iter()
                .zip(prior.body.native().relations())
            {
                StructuralWalkObservation::checkpoint(context)?;
                if a != b {
                    return Err(WalkRecordDenial::Conflict);
                }
            }
        }
        let key = WorthQueryApplicationFactKey::StructuralWalk(
            prior.map_or_else(|| body.key().clone(), |p| p.body.key().clone()),
        );
        let registry = if prior.is_none() {
            insertion_bytes::<(StructuralWalkKey, Issued)>(self.walks.values.len())
                .ok_or(Cause::ChargedBytesOverflow)?
        } else {
            0
        };
        let attribution = match self.reading {
            None if !self.handler.contains(&key) => {
                insertion_bytes::<WorthQueryApplicationFactKey>(self.handler.len())
            }
            None => Some(0),
            Some(read) => self.computation.structural_read_bytes(&key, read),
        }
        .ok_or(Cause::ChargedBytesOverflow)?;
        let bytes = self
            .walks
            .bytes
            .checked_add(registry)
            .and_then(|n| n.checked_add(attribution))
            .ok_or(Cause::ChargedBytesOverflow)?;
        if bytes != self.walks.bytes {
            if self.walks.memory.is_none() {
                self.walks.memory = super::super::super::walk::memory::reserve(
                    execution,
                    context.has_accounting(),
                    bytes,
                )?;
            } else {
                super::super::super::walk::memory::resize(&mut self.walks.memory, bytes)?;
            }
        }
        if prior.is_none() {
            self.walks.maximum_roots = self.walks.maximum_roots.max(body.key().roots().len());
            self.walks.values.insert(
                body.key().clone(),
                Issued {
                    cookie: body.cookie(),
                    body,
                    admission,
                    contracts,
                },
            );
        }
        self.walks.bytes = bytes;
        self.record(key, DecisionReadOutcome::Observed);
        Ok(())
    }
}
