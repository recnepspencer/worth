//! Closed acquisition postures preserve their real Native proof owners.
use super::super::{
    applied_affinity::AppliedSourceAffinity, ManagedIndexPublishedSourceRoot,
    ManagedIndexSourceRows,
};
use super::*;
use crate::derived_relation::definition::DerivedRelationHandle;
use crate::derived_relation::read::DerivedRelationData;
use crate::derived_relation::DerivedRelationReadShape;
use crate::derived_relation::PinnedDerivedRelationRead;
use crate::derived_relation::PreparedDerivedRelationRead;
use crate::visibility::materialization::read_records::projection::VisibilityProjectionView;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(in crate::validation::candidate_source) enum SourceProjectionBacking<'a> {
    Invariant {
        relations: &'a StructuralRelationView<'a>,
        aspects: &'a StructuralAspectStateView<'a>,
        audience: AdmittedManagedIndexSourceAudience,
        interpretation: &'a super::super::NativeManagedSourceInterpretation,
        before: bool,
    },
    Applied {
        projection: &'a VisibilityProjectionView,
        affinity: &'a Arc<AppliedSourceAffinity>,
        execution: ExecutionRequest<'a, 'a>,
        memory: RefCell<Option<ExecutionMemoryReservation>>,
        bytes: Cell<u64>,
    },
    Selected {
        projection: &'a VisibilityProjectionView,
        expected: &'a Arc<ManagedIndexPublishedSourceRoot>,
        execution: ExecutionRequest<'a, 'a>,
        memory: RefCell<Option<ExecutionMemoryReservation>>,
        bytes: Cell<u64>,
    },
}
/// Escaping derived backing retains its original admitted payer, not a request.
/// This type grants no graph, audience or publication authority.
pub(in crate::validation::candidate_source) enum SourceProjectionRetention {
    Invariant {
        _meter: crate::validation::custom_rule::CustomInvariantWorkMeter,
    },
    Request {
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
    Applied {
        read: PinnedDerivedRelationRead,
        affinity: Arc<AppliedSourceAffinity>,
    },
    Invariant(PreparedDerivedRelationRead),
    Selected {
        read: PinnedDerivedRelationRead,
        entry: Arc<super::super::ManagedIndexPublishedSourceRoot>,
        root: Arc<crate::branch::root::RelationalBranchRoot>,
    },
}
impl ManagedIndexSourceProjection<'_> {
    pub(super) fn physical_projection(
        &self,
    ) -> Option<(&VisibilityProjectionView, &ExecutionRequest<'_, '_>)> {
        match &self.backing {
            SourceProjectionBacking::Selected {
                projection,
                execution,
                ..
            } => Some((projection, execution)),
            SourceProjectionBacking::Applied {
                projection,
                execution,
                ..
            } => Some((projection, execution)),
            SourceProjectionBacking::Invariant { .. } => None,
        }
    }
    pub(in crate::validation::candidate_source) fn scope_identity(
        &self,
        scope: EntityId,
    ) -> Result<Option<crate::derived_relation::DerivedRelationReadIdentity>, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => {
                Ok(Some(relations.derived_source_identity(
                    &self.program.0.path,
                    scope,
                    DerivedRelationReadShape::Adjacency { reverse: false },
                )?))
            }
            SourceProjectionBacking::Selected { .. } | SourceProjectionBacking::Applied { .. } => {
                Ok(None)
            }
        }
    }

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
            SourceProjectionBacking::Selected { execution, .. }
            | SourceProjectionBacking::Applied { execution, .. } => {
                self.checkpoint(0)?;
                let memory = execution
                    .reserve_retained_memory(bytes)
                    .map_err(|cause| Denial::Request(cause.into()))?;
                Ok(SourceProjectionRetention::Request { _memory: memory })
            }
        }
    }

    pub(crate) fn checkpoint(&self, units: u64) -> Result<(), Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { relations, .. } => relations
                .read_budget()
                .checkpoint(units)
                .map_err(Denial::Structural),
            SourceProjectionBacking::Selected { execution, .. }
            | SourceProjectionBacking::Applied { execution, .. } => execution
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
            }
            | SourceProjectionBacking::Applied {
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
            SourceProjectionBacking::Selected { .. } | SourceProjectionBacking::Applied { .. } => {
                let (projection, execution) = self
                    .physical_projection()
                    .ok_or(Definition::ForeignObservation)?;
                self.checkpoint(
                    projection
                        .exact_entity_state_read_work_bound()
                        .ok_or(Definition::RecordUnavailable)?,
                )?;
                projection
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
            SourceProjectionBacking::Applied {
                projection,
                affinity,
                execution,
                ..
            } => PreparedBody::Applied {
                read: PinnedDerivedRelationRead::acquire_projected(
                    projection,
                    execution,
                    path,
                    root.into(),
                    shape,
                    None,
                )?,
                affinity: Arc::clone(affinity),
            },
            SourceProjectionBacking::Invariant { relations, .. } => {
                PreparedBody::Invariant(relations.prepare_derived_source(path, root, shape)?)
            }
            SourceProjectionBacking::Selected {
                projection,
                expected,
                execution,
                ..
            } => PreparedBody::Selected {
                read: PinnedDerivedRelationRead::acquire_projected(
                    projection,
                    execution,
                    path,
                    root.into(),
                    shape,
                    None,
                )?,
                entry: Arc::clone(expected),
                root: Arc::clone(
                    projection
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
                SourceProjectionBacking::Applied {
                    affinity: expected, ..
                },
                PreparedBody::Applied { read, affinity },
            ) if Arc::ptr_eq(expected, affinity) => Ok(read.data()),
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
                SourceProjectionBacking::Selected {
                    projection,
                    expected,
                    ..
                },
                PreparedBody::Selected { read, entry, root },
            ) if Arc::ptr_eq(expected, entry)
                && projection
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
        project: impl FnOnce(Observation<'_>) -> T,
    ) -> Result<T, Denial> {
        match &self.backing {
            SourceProjectionBacking::Invariant { .. } => {
                self.field_at(row, kind, field).map(project)
            }
            SourceProjectionBacking::Selected { .. } | SourceProjectionBacking::Applied { .. } => {
                let (projection, execution) = self
                    .physical_projection()
                    .ok_or(Definition::ForeignObservation)?;
                self.checkpoint(
                    projection
                        .exact_entity_state_read_work_bound()
                        .ok_or(Definition::RecordUnavailable)?,
                )?;
                projection
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
fn record_denial(
    cause: crate::visibility::materialization::read_records::projection::RelationalBorrowedRecordReadDenial,
) -> Denial {
    match cause {
        crate::visibility::materialization::read_records::projection::RelationalBorrowedRecordReadDenial::Execution(cause) => {
            Denial::Request(cause)
        }
        crate::visibility::materialization::read_records::projection::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
            Definition::RecordUnavailable.into()
        }
    }
}

mod applied_rows;
pub(in crate::validation::candidate_source) use applied_rows::applied_rows;

pub(crate) fn selected_rows<'a>(
    projection: &'a VisibilityProjectionView,
    expected: &'a Arc<ManagedIndexPublishedSourceRoot>,
    execution: ExecutionRequest<'a, 'a>,
) -> ManagedIndexSourceRows<'a> {
    let association = &expected.association;
    ManagedIndexSourceRows {
        scope: association.scope,
        projection: ManagedIndexSourceProjection {
            program: &association.program,
            backing: SourceProjectionBacking::Selected {
                projection,
                expected,
                execution,
                memory: RefCell::new(None),
                bytes: Cell::new(0),
            },
        },
    }
}
