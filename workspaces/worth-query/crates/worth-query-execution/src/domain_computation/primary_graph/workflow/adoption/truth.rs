//! One bounded owner-truth read shared by every workflow adoption inventory
//! step, so the whole inventory spends a single `maximum_selection_work`
//! allowance and refuses rather than truncating.
//!
//! Every read goes through the selected branch's own root, so a fork's
//! inventory sees the fork's writes and never its parent's later ones.

use std::cell::Cell;
use std::collections::BTreeSet;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::runtime::{
    ProjectionAspectScope, RelationKindTruthReadDenial, RelationReadRecord,
    VisibilityProjectionView,
};
use worth_relational::facade::storage::RecordLifecycleState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowAdoptionReadDenial {
    WorkLimitExceeded {
        consumed_work_units: usize,
    },
    UnreadableRelationSlot {
        partition_id: PartitionId,
        slot: usize,
    },
    UnreadableEntity {
        entity: EntityId,
    },
}

pub(in crate::domain_computation::primary_graph) struct WorkflowAdoptionTruth<'runtime> {
    view: VisibilityProjectionView<'runtime>,
    maximum_work_units: usize,
    // Shared with the entities it lends out, which charge each field read.
    consumed_work_units: Cell<usize>,
}

impl<'runtime> WorkflowAdoptionTruth<'runtime> {
    pub(in crate::domain_computation::primary_graph) const fn new(
        view: VisibilityProjectionView<'runtime>,
        maximum_work_units: usize,
    ) -> Self {
        Self {
            view,
            maximum_work_units,
            consumed_work_units: Cell::new(0),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn consumed_work_units(&self) -> usize {
        self.consumed_work_units.get()
    }

    fn remaining(&self) -> usize {
        self.maximum_work_units
            .saturating_sub(self.consumed_work_units.get())
    }

    fn exhausted(&self, attempted: usize) -> WorkflowAdoptionReadDenial {
        WorkflowAdoptionReadDenial::WorkLimitExceeded {
            consumed_work_units: self.consumed_work_units.get().saturating_add(attempted),
        }
    }

    fn spend(&self, units: usize) {
        self.consumed_work_units
            .set(self.consumed_work_units.get().saturating_add(units));
    }

    /// One point read, refused when the allowance is spent.
    fn charge_one(&self) -> Result<(), WorkflowAdoptionReadDenial> {
        if self.remaining() == 0 {
            return Err(self.exhausted(1));
        }
        self.spend(1);
        Ok(())
    }

    /// Every live relation of one kind on the branch.
    pub(in crate::domain_computation::primary_graph) fn relations_of_kind(
        &mut self,
        kind: KindId,
    ) -> Result<Vec<RelationReadRecord>, WorkflowAdoptionReadDenial> {
        let read = self
            .view
            .bounded_relations_of_kind(kind, self.remaining())
            .map_err(|denial| match denial {
                RelationKindTruthReadDenial::WorkLimitExceeded(limit) => {
                    self.exhausted(limit.consumed_work_units())
                }
                RelationKindTruthReadDenial::UnreadableRelationSlot { partition_id, slot } => {
                    WorkflowAdoptionReadDenial::UnreadableRelationSlot { partition_id, slot }
                }
            })?;
        self.spend(read.work_units());
        Ok(read.into_records())
    }

    /// Live outgoing relations of one kind from `entity`.
    pub(in crate::domain_computation::primary_graph) fn outgoing(
        &mut self,
        entity: EntityId,
        kind: KindId,
    ) -> Result<Vec<RelationReadRecord>, WorkflowAdoptionReadDenial> {
        let read = self
            .view
            .bounded_outgoing_relations_for_frontier(
                &BTreeSet::from([entity]),
                kind,
                self.remaining(),
            )
            .map_err(|limit| self.exhausted(limit.consumed_work_units()))?;
        self.spend(read.work_units());
        Ok(read.into_records())
    }

    /// The single live outgoing endpoint of one relation kind.
    pub(in crate::domain_computation::primary_graph) fn single_target(
        &mut self,
        entity: EntityId,
        kind: KindId,
    ) -> Result<EntityId, WorkflowAdoptionReadDenial> {
        match self.outgoing(entity, kind)?.as_slice() {
            [relation] => Ok(relation.target),
            _ => Err(WorkflowAdoptionReadDenial::UnreadableEntity { entity }),
        }
    }

    /// One live entity of the expected kind, charged one unit of work.
    pub(in crate::domain_computation::primary_graph) fn entity(
        &mut self,
        entity: EntityId,
        kind: KindId,
    ) -> Result<WorkflowAdoptionEntity<'_, 'runtime>, WorkflowAdoptionReadDenial> {
        self.charge_one()?;
        self.view
            .entity_record_with_projection_scope(entity, ProjectionAspectScope::empty(), |record| {
                (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live)
                    .then_some(())
            })
            .map(|()| WorkflowAdoptionEntity {
                truth: self,
                entity,
            })
            .ok_or(WorkflowAdoptionReadDenial::UnreadableEntity { entity })
    }
}

/// A live entity on the branch, whose fields are read from the same root,
/// each charged one unit of the same allowance.
pub(in crate::domain_computation::primary_graph) struct WorkflowAdoptionEntity<'truth, 'runtime> {
    truth: &'truth WorkflowAdoptionTruth<'runtime>,
    entity: EntityId,
}

impl WorkflowAdoptionEntity<'_, '_> {
    fn value(
        &self,
        locator: &AspectFieldLocator,
    ) -> Result<Option<AspectValue>, WorkflowAdoptionReadDenial> {
        let field = locator
            .field_path()
            .fields()
            .first()
            .ok_or_else(|| self.unreadable())?;
        self.truth.charge_one()?;
        let aspect = locator.aspect().aspect_key();
        let scope = ProjectionAspectScope::whole_aspects([aspect.clone()]);
        // An absent aspect holds no field; a scalar one is not this layout.
        self.truth
            .view
            .entity_record_with_projection_scope(self.entity, scope, |record| {
                Some(match record.struct_aspect_value(aspect) {
                    Some(fields) => Ok(fields.get(field).cloned()),
                    None if record.aspect_value(aspect).is_some() => Err(()),
                    None => Ok(None),
                })
            })
            .and_then(Result::ok)
            .ok_or_else(|| self.unreadable())
    }

    fn unreadable(&self) -> WorkflowAdoptionReadDenial {
        WorkflowAdoptionReadDenial::UnreadableEntity {
            entity: self.entity,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn optional_text(
        &self,
        locator: &AspectFieldLocator,
    ) -> Result<Option<String>, WorkflowAdoptionReadDenial> {
        match self.value(locator)? {
            None => Ok(None),
            Some(AspectValue::String(InternedString::Raw(text))) => Ok(Some(text)),
            Some(_) => Err(self.unreadable()),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn text(
        &self,
        locator: &AspectFieldLocator,
    ) -> Result<String, WorkflowAdoptionReadDenial> {
        self.optional_text(locator)?
            .ok_or_else(|| self.unreadable())
    }

    pub(in crate::domain_computation::primary_graph) fn u64(
        &self,
        locator: &AspectFieldLocator,
    ) -> Result<u64, WorkflowAdoptionReadDenial> {
        match self.value(locator)? {
            Some(AspectValue::UInt64(value)) => Ok(value),
            _ => Err(self.unreadable()),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn bool(
        &self,
        locator: &AspectFieldLocator,
    ) -> Result<bool, WorkflowAdoptionReadDenial> {
        match self.value(locator)? {
            Some(AspectValue::Bool(value)) => Ok(value),
            _ => Err(self.unreadable()),
        }
    }
}
