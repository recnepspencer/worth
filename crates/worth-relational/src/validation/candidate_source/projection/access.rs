//! Closed acquisition postures preserve their real Native proof owners.
use super::*;
use crate::derived_relation::{
    DerivedRelationData, DerivedRelationHandle, DerivedRelationReadShape,
    PinnedDerivedRelationRead, PreparedDerivedRelationRead,
};
use crate::mvcc::ManagedIndexSourceRebuild;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(crate) enum SourceProjectionBacking<'a> {
    Invariant {
        relations: &'a StructuralRelationView<'a>,
        aspects: &'a StructuralAspectStateView<'a>,
        audience: AdmittedManagedIndexSourceAudience,
        interpretation: &'a super::super::NativeManagedSourceInterpretation,
        before: bool,
    },
    Selected {
        loan: &'a ManagedIndexSourceRebuild,
        execution: ExecutionRequest<'a, 'a>,
        memory: RefCell<Option<ExecutionMemoryReservation>>,
        bytes: Cell<u64>,
    },
}
/// Escaping derived backing retains its original admitted payer, not a request.
/// This type grants no graph, audience or publication authority.
pub(in crate::validation::candidate_source) enum SourceProjectionRetention {
    Invariant {
        _meter: crate::validation::CustomInvariantWorkMeter,
    },
    Selected {
        _memory: Option<ExecutionMemoryReservation>,
    },
}

/// Complete relation support is consumed only by its actual originating loan.
pub struct PreparedManagedIndexRelationRead {
    body: PreparedBody,
    path: DerivedRelationHandle,
    root: EntityId,
    shape: DerivedRelationReadShape,
}
enum PreparedBody {
    Invariant(PreparedDerivedRelationRead),
    Selected {
        read: PinnedDerivedRelationRead,
        entry: Arc<super::super::ManagedIndexPublishedSourceRoot>,
        root: Arc<crate::branch::RelationalBranchRoot>,
    },
}
impl ManagedIndexSourceProjection<'_> {
    pub(in crate::validation::candidate_source) fn owned_retention(
        &self,
        bytes: u64,
    ) -> Result<SourceProjectionRetention, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => {
                let budget = relations.read_budget();
                budget.claim_retained(bytes).map_err(Denial::Structural)?;
                Ok(SourceProjectionRetention::Invariant {
                    _meter: budget.retained_payer(),
                })
            }
            SourceProjectionBacking::Selected { execution, .. } => {
                self.checkpoint(0)?;
                let memory = execution
                    .reserve_retained_memory(bytes)
                    .map_err(|cause| Denial::Request(cause.into()))?;
                Ok(SourceProjectionRetention::Selected { _memory: memory })
            }
        }
    }

    pub(crate) fn checkpoint(&self, units: u64) -> Result<(), Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => relations
                .read_budget()
                .checkpoint(units)
                .map_err(Denial::Structural),
            SourceProjectionBacking::Selected { execution, .. } => execution
                .run_native_with_checkpoint(|context| {
                    crate::execution::request_stage::checkpoint(context, units)
                })
                .map(|_| ())
                .map_err(|cause| Denial::Request(cause.into())),
        }
    }
    pub(crate) fn claim_retained(&self, bytes: u64) -> Result<(), Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => relations
                .read_budget()
                .claim_retained(bytes)
                .map_err(Denial::Structural),
            SourceProjectionBacking::Selected {
                execution,
                memory,
                bytes: total,
                ..
            } => {
                let next = total
                    .get()
                    .checked_add(bytes)
                    .ok_or(Definition::WorkBudgetExceeded)?;
                let mut memory = memory.borrow_mut();
                if let Some(grant) = &mut *memory {
                    grant.resize(next).map_err(|cause| {
                        Denial::Request(crate::execution::denial::lease_denial(
                            worth_execution::LeaseDenial::MemoryExhausted(cause),
                        ))
                    })?;
                } else {
                    *memory = execution
                        .reserve_retained_memory(next)
                        .map_err(|cause| Denial::Request(cause.into()))?;
                }
                total.set(next);
                Ok(())
            }
        }
    }
    pub(crate) fn entity_kind(
        &self,
        row: EntityId,
    ) -> Result<crate::identity::data::KindId, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => relations
                .entity_kind(row)
                .map_err(|cause| Denial::Structural(relations.read_budget().refusal(cause))),
            SourceProjectionBacking::Selected {
                loan, execution, ..
            } => {
                self.checkpoint(
                    loan.projection
                        .exact_entity_state_read_work_bound()
                        .ok_or(Definition::RecordUnavailable)?,
                )?;
                loan.projection
                    .read_exact_entity_state(execution, row, |metadata, _| metadata)
                    .map_err(record_denial)?
                    .filter(|record| record.entity_id == row)
                    .map(|record| record.kind_id)
                    .ok_or(Definition::RecordUnavailable.into())
            }
        }
    }
    pub(crate) fn prepare(
        &self,
        path: &DerivedRelationHandle,
        root: EntityId,
        shape: DerivedRelationReadShape,
    ) -> Result<PreparedManagedIndexRelationRead, Denial> {
        let body = match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => {
                PreparedBody::Invariant(relations.prepare_derived_source(path, root, shape)?)
            }
            SourceProjectionBacking::Selected {
                loan, execution, ..
            } => PreparedBody::Selected {
                read: loan
                    .projection
                    .derived_source(execution, path, root.into(), shape, None)?,
                entry: Arc::clone(&loan.expected),
                root: Arc::clone(
                    loan.projection
                        .selected_root()
                        .ok_or(Definition::RecordUnavailable)?,
                ),
            },
        };
        Ok(PreparedManagedIndexRelationRead {
            body,
            path: path.clone(),
            root,
            shape,
        })
    }
    pub(crate) fn read<'a>(
        &'a self,
        read: &'a PreparedManagedIndexRelationRead,
        path: &DerivedRelationHandle,
        root: EntityId,
        shape: DerivedRelationReadShape,
    ) -> Result<&'a DerivedRelationData, Denial> {
        if read.path != *path || read.root != root || read.shape != shape {
            return Err(Definition::ForeignObservation.into());
        }
        match (&self.backing, &read.body) {
            (
                SourceProjectionBacking::Invariant { relations, .. },
                PreparedBody::Invariant(read),
            ) => {
                if read.identity() != &relations.derived_source_identity(path, root, shape)? {
                    return Err(Definition::ForeignObservation.into());
                }
                relations.read_prepared_derived_source(read, shape)
            }
            (
                SourceProjectionBacking::Selected { loan, .. },
                PreparedBody::Selected { read, entry, root },
            ) if Arc::ptr_eq(&loan.expected, entry)
                && loan
                    .projection
                    .selected_root()
                    .is_some_and(|actual| Arc::ptr_eq(actual, root)) =>
            {
                Ok(read.data())
            }
            _ => Err(Definition::ForeignObservation.into()),
        }
    }
    pub(crate) fn with_field_at<T>(
        &self,
        row: EntityId,
        kind: crate::identity::data::KindId,
        field: &super::super::ManagedIndexSourceField,
        project: impl FnOnce(Option<&AspectValue>) -> T,
    ) -> Result<T, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { .. } => {
                self.field_at(row, kind, field).map(project)
            }
            SourceProjectionBacking::Selected {
                loan, execution, ..
            } => {
                self.checkpoint(
                    loan.projection
                        .exact_entity_state_read_work_bound()
                        .ok_or(Definition::RecordUnavailable)?,
                )?;
                loan.projection
                    .read_exact_entity_state(execution, row, |metadata, state| {
                        if metadata.entity_id != row || metadata.kind_id != kind {
                            return Err(Definition::InvalidMemberDomain.into());
                        }
                        field_value(
                            state.and_then(|state| state.get(field.locator.aspect().aspect_key())),
                            field,
                        )
                        .map(project)
                    })
                    .map_err(record_denial)?
                    .ok_or(Definition::RecordUnavailable)?
            }
        }
    }
}
fn record_denial(cause: crate::runtime::RelationalBorrowedRecordReadDenial) -> Denial {
    match cause {
        crate::runtime::RelationalBorrowedRecordReadDenial::Execution(cause) => {
            Denial::Request(cause)
        }
        crate::runtime::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
            Definition::RecordUnavailable.into()
        }
    }
}
impl ManagedIndexSourceRebuild {
    /// Each operation reborrows this caller's original request. The retained loan
    /// contains neither a request nor an invariant audience.
    pub fn rows<'a>(
        &'a self,
        execution: ExecutionRequest<'a, 'a>,
    ) -> super::super::ManagedIndexSourceRows<'a> {
        super::super::ManagedIndexSourceRows {
            scope: self.scope(),
            projection: ManagedIndexSourceProjection {
                program: self.program(),
                backing: SourceProjectionBacking::Selected {
                    loan: self,
                    execution,
                    memory: RefCell::new(None),
                    bytes: Cell::new(0),
                },
            },
        }
    }
}
