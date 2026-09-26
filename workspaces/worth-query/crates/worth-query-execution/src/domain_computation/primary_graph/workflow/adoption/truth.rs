//! One bounded owner-truth read shared by every workflow adoption inventory
//! step, so the whole inventory spends a single `maximum_selection_work`
//! allowance and refuses rather than truncating.

use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, ContractValidatedAspectValueView, InternedString,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, VersionId};
use worth_relational::facade::runtime::{
    EntityReadRecord, RelationKindTruthReadDenial, RelationReadRecord, RelationalRuntime,
};

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
    runtime: &'runtime RelationalRuntime,
    version: VersionId,
    maximum_work_units: usize,
    consumed_work_units: usize,
}

impl<'runtime> WorkflowAdoptionTruth<'runtime> {
    pub(in crate::domain_computation::primary_graph) const fn new(
        runtime: &'runtime RelationalRuntime,
        version: VersionId,
        maximum_work_units: usize,
    ) -> Self {
        Self {
            runtime,
            version,
            maximum_work_units,
            consumed_work_units: 0,
        }
    }

    pub(in crate::domain_computation::primary_graph) const fn consumed_work_units(&self) -> usize {
        self.consumed_work_units
    }

    const fn remaining(&self) -> usize {
        self.maximum_work_units
            .saturating_sub(self.consumed_work_units)
    }

    fn exhausted(&self, attempted: usize) -> WorkflowAdoptionReadDenial {
        WorkflowAdoptionReadDenial::WorkLimitExceeded {
            consumed_work_units: self.consumed_work_units.saturating_add(attempted),
        }
    }

    /// Every live relation of one kind visible at the inventory version.
    pub(in crate::domain_computation::primary_graph) fn relations_of_kind(
        &mut self,
        kind: KindId,
    ) -> Result<Vec<RelationReadRecord>, WorkflowAdoptionReadDenial> {
        let read = self
            .runtime
            .read_truth()
            .bounded_visible_relations_of_kind(kind, self.version, self.remaining())
            .map_err(|denial| match denial {
                RelationKindTruthReadDenial::WorkLimitExceeded(limit) => {
                    self.exhausted(limit.consumed_work_units())
                }
                RelationKindTruthReadDenial::UnreadableRelationSlot { partition_id, slot } => {
                    WorkflowAdoptionReadDenial::UnreadableRelationSlot { partition_id, slot }
                }
            })?;
        self.consumed_work_units = self.consumed_work_units.saturating_add(read.work_units());
        Ok(read.into_records())
    }

    /// Live outgoing relations of one kind from `entity`.
    pub(in crate::domain_computation::primary_graph) fn outgoing(
        &mut self,
        entity: EntityId,
        kind: KindId,
    ) -> Result<Vec<RelationReadRecord>, WorkflowAdoptionReadDenial> {
        let read = self
            .runtime
            .read_truth()
            .bounded_outgoing_relations_of_kind_at_version(
                entity,
                kind,
                self.version,
                self.remaining(),
            )
            .map_err(|limit| self.exhausted(limit.consumed_work_units()))?;
        self.consumed_work_units = self.consumed_work_units.saturating_add(read.work_units());
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
    ) -> Result<WorkflowAdoptionEntity, WorkflowAdoptionReadDenial> {
        if self.remaining() == 0 {
            return Err(self.exhausted(1));
        }
        self.consumed_work_units += 1;
        self.runtime
            .read_truth()
            .visible_entity_at_version(entity, self.version)
            .filter(|record| record.kind.kind_id == kind)
            .map(|record| WorkflowAdoptionEntity { entity, record })
            .ok_or(WorkflowAdoptionReadDenial::UnreadableEntity { entity })
    }
}

pub(in crate::domain_computation::primary_graph) struct WorkflowAdoptionEntity {
    entity: EntityId,
    record: EntityReadRecord,
}

impl WorkflowAdoptionEntity {
    fn value(
        &self,
        locator: &AspectFieldLocator,
    ) -> Result<Option<AspectValue>, WorkflowAdoptionReadDenial> {
        let unreadable = WorkflowAdoptionReadDenial::UnreadableEntity {
            entity: self.entity,
        };
        let Some(state) = self.record.authoritative_aspect_state.as_ref() else {
            return Err(unreadable);
        };
        let Some(aspect) = state.get(locator.aspect().aspect_key()) else {
            return Ok(None);
        };
        let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
            return Err(unreadable);
        };
        let field = locator.field_path().fields().first().ok_or(unreadable)?;
        Ok(fields.get(field).cloned())
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
