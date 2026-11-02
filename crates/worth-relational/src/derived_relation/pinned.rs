//! The same recipe evaluator over one actual immutable published Native root.
use super::evaluation::SourceRead;
use super::{DerivedRelationAdjacency as Adjacency, DerivedRelationData as Data};
use super::{DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle};
use super::{DerivedRelationReadDenial as Denial, DerivedRelationReads as Reads};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, KindId};
use crate::runtime::RelationalPinnedRead;
use crate::storage::data::RecordLifecycleState;
use crate::validation::data::StructuralRelationRecord;
use crate::visibility::materialization::read_records::{
    RelationalAdjacencyDirection as Direction, RelationalAdjacencyVisit as Visit,
    VisibilityProjectionView,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelContext};

mod acquisition;
mod scope;
pub use scope::{PinnedDerivedRelationDenial, PinnedDerivedRelationScope};

/// Descriptive complete facts retain their actual request backing. They grant
/// no later source, decision, mutation or disclosure authority.
pub struct PinnedDerivedRelationRead {
    data: Data,
    steps: Vec<super::DerivedRelationWalkStep>,
    shape: super::DerivedRelationReadShape,
    work: usize,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PinnedDerivedRelationRead {
    pub fn data(&self) -> &Data {
        &self.data
    }
    /// Actual checkpointed acquisition, navigation and evaluation work.
    pub const fn work_units(&self) -> usize {
        self.work
    }
    pub fn steps(&self) -> &[super::DerivedRelationWalkStep] {
        &self.steps
    }
    pub const fn shape(&self) -> super::DerivedRelationReadShape {
        self.shape
    }
}

struct PinnedSource<'view, 'control, 'request, 'context, 'worker, 'scope> {
    projection: &'view VisibilityProjectionView,
    execution: ExecutionRequest<'request, 'request>,
    context: &'context mut MapKernelContext<'worker, 'scope>,
    reads: Reads,
    memory: Option<ExecutionMemoryReservation>,
    bytes: u64,
    work: usize,
    maximum_work: Option<usize>,
    control: Option<&'control dyn Fn() -> Result<(), Cause>>,
}

impl SourceRead for PinnedSource<'_, '_, '_, '_, '_, '_> {
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial> {
        checkpoint(
            self.context,
            &mut self.work,
            self.maximum_work,
            self.control,
            units as u64,
        )
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Denial> {
        retain(&mut self.memory, &mut self.bytes, bytes as u64)
    }
    fn check_scratch_peak(&mut self, bytes: usize) -> Result<(), Denial> {
        self.execution
            .check_live()
            .map_err(|cause| Denial::Request(cause.into()))?;
        self.checkpoint(0)?;
        let peak = self
            .bytes
            .checked_add(bytes as u64)
            .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
        if let Some(memory) = &mut self.memory
            && peak > memory.bytes()
        {
            memory.resize(peak).map_err(|cause| {
                Denial::Request(crate::execution::denial::lease_denial(
                    worth_execution::LeaseDenial::MemoryExhausted(cause),
                ))
            })?;
        }
        Ok(())
    }
    fn entity_kind(&mut self, id: EntityId) -> Result<KindId, Denial> {
        self.checkpoint(
            self.projection
                .exact_entity_state_read_work_bound()
                .ok_or(DefinitionDenial::RecordUnavailable)? as usize,
        )?;
        let entity = self
            .projection
            .read_exact_entity_state(&self.execution, id, |metadata, _| metadata)
            .map_err(|cause| match cause {
                crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                    Denial::Definition(DefinitionDenial::RecordUnavailable)
                }
                crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {
                    Denial::Request(cause)
                }
            })?
            .ok_or(DefinitionDenial::RecordUnavailable)?;
        if entity.entity_id != id || entity.lifecycle != RecordLifecycleState::Live {
            return Err(DefinitionDenial::RecordUnavailable.into());
        }
        self.reads.entity_records += 1;
        Ok(entity.kind_id)
    }
    fn adjacency(&mut self, probe: Adjacency) -> Result<Vec<StructuralRelationRecord>, Denial> {
        let mut records = Vec::new();
        let direction = if probe.outgoing {
            Direction::Outgoing
        } else {
            Direction::Incoming
        };
        let reads = &mut self.reads;
        let context = &mut self.context;
        let projection = self.projection;
        let memory = &mut self.memory;
        let bytes = &mut self.bytes;
        let work = &mut self.work;
        let maximum_work = self.maximum_work;
        let control = self.control;
        projection
            .try_visit_adjacency_ids(
                &self.execution,
                probe.anchor,
                probe.relation,
                direction,
                |event| {
                    match event {
                        Visit::Prepare {
                            work: units,
                            bytes: temporary,
                        } => {
                            checkpoint(context, work, maximum_work, control, units)?;
                            retain(memory, bytes, temporary)?;
                        }
                        Visit::List => checkpoint(context, work, maximum_work, control, 1)?,
                        Visit::Acquired => reads.adjacency_lists += 1,
                        Visit::Relation(id) => {
                            checkpoint(
                                context,
                                work,
                                maximum_work,
                                control,
                                projection
                                    .exact_relation_metadata_read_work_bound()
                                    .ok_or(DefinitionDenial::RecordUnavailable)?,
                            )?;
                            let record = projection
                                .read_exact_relation_metadata(&self.execution, id)
                                .map_err(|cause| match cause {crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired=>Denial::Definition(DefinitionDenial::RecordUnavailable),crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause)=>Denial::Request(cause)})?;
                            if let Some(record) = record {
                                reads.relation_records += 1;
                                if record.lifecycle == RecordLifecycleState::Live
                                    && record.kind_id == probe.relation
                                {
                                    retain(
                                        memory,
                                        bytes,
                                        size_of::<StructuralRelationRecord>() as u64,
                                    )?;
                                    records.push(StructuralRelationRecord {
                                        relation_id: record.relation_id,
                                        kind_id: record.kind_id,
                                        source: record.source,
                                        target: record.target,
                                    });
                                }
                            }
                        }
                    }
                    Ok::<_, Denial>(())
                },
            )
            .map_err(Denial::Request)??;
        Ok(records)
    }
    fn reads(&self) -> Reads {
        self.reads
    }
}

fn checkpoint(
    context: &mut MapKernelContext<'_, '_>,
    work: &mut usize,
    maximum: Option<usize>,
    control: Option<&dyn Fn() -> Result<(), Cause>>,
    units: u64,
) -> Result<(), Denial> {
    let result = if context.has_accounting() {
        context.checkpoint(units)
    } else {
        context.poll()
    };
    result.map_err(|cause| Denial::Request(cause.into()))?;
    let units = usize::try_from(units).map_err(|_| Denial::Request(Cause::WorkCounterOverflow))?;
    let next = work
        .checked_add(units)
        .ok_or(Denial::Request(Cause::WorkCounterOverflow))?;
    // This work was admitted by the original checkpoint even when an extra
    // caller control or local limit now refuses the result.
    *work = next;
    if let Some(control) = control {
        control().map_err(Denial::Request)?;
    }
    if maximum.is_some_and(|maximum| next > maximum) {
        return Err(DefinitionDenial::WorkBudgetExceeded.into());
    }
    Ok(())
}
fn retain(
    memory: &mut Option<ExecutionMemoryReservation>,
    total: &mut u64,
    bytes: u64,
) -> Result<(), Denial> {
    let next = total
        .checked_add(bytes)
        .ok_or(Denial::Request(Cause::ChargedBytesOverflow))?;
    if let Some(memory) = memory {
        memory.resize(next).map_err(|cause| {
            Denial::Request(crate::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ))
        })?;
    }
    *total = next;
    Ok(())
}
