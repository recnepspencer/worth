//! Bounded owner-truth inventory for a program change on one selected branch.

use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
use worth_relational::facade::{
    identity::{EntityId, VersionId},
    runtime::{RelationKindTruthReadDenial, RelationalRuntime},
};

use super::super::schema::WorthQueryWorkflowLayout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowAdoptionInventoryDenial {
    WorkLimitExceeded {
        consumed_work_units: usize,
    },
    UnreadableRelationSlot {
        partition_id: worth_relational::facade::identity::PartitionId,
        slot: usize,
    },
    UnreadableInstance {
        instance: EntityId,
    },
}

pub(in crate::domain_computation::primary_graph) struct WorkflowAdoptionInventory {
    instances: Box<[EntityId]>,
    work_units: usize,
}

impl WorkflowAdoptionInventory {
    pub(in crate::domain_computation::primary_graph) fn instances(&self) -> &[EntityId] {
        &self.instances
    }

    pub(in crate::domain_computation::primary_graph) const fn work_units(&self) -> usize {
        self.work_units
    }
}

pub(in crate::domain_computation::primary_graph) fn inventory_for_adoption(
    runtime: &RelationalRuntime,
    layout: &WorthQueryWorkflowLayout,
    version: VersionId,
    branch_occurrence: u64,
    maximum_work_units: usize,
) -> Result<WorkflowAdoptionInventory, WorkflowAdoptionInventoryDenial> {
    let read = runtime.read_truth();
    let live = read
        .bounded_visible_relations_of_kind(
            layout.live_instance_lineage_relation,
            version,
            maximum_work_units,
        )
        .map_err(|denial| match denial {
            RelationKindTruthReadDenial::WorkLimitExceeded(limit) => {
                WorkflowAdoptionInventoryDenial::WorkLimitExceeded {
                    consumed_work_units: limit.consumed_work_units(),
                }
            }
            RelationKindTruthReadDenial::UnreadableRelationSlot { partition_id, slot } => {
                WorkflowAdoptionInventoryDenial::UnreadableRelationSlot { partition_id, slot }
            }
        })?;
    let mut work_units = live.work_units();
    let mut instances = Vec::new();
    for membership in live.into_records() {
        if work_units == maximum_work_units {
            return Err(WorkflowAdoptionInventoryDenial::WorkLimitExceeded {
                consumed_work_units: work_units,
            });
        }
        work_units += 1;
        let instance = membership.source;
        let record = read
            .visible_entity_at_version(instance, version)
            .filter(|record| record.kind.kind_id == layout.instance.entity_kind)
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        let state = record
            .authoritative_aspect_state
            .as_ref()
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        let value = state
            .get(layout.instance.program_revision.aspect().aspect_key())
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
            return Err(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance });
        };
        let protocol_field = layout
            .instance
            .protocol_version
            .field_path()
            .fields()
            .first()
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        if fields.get(protocol_field)
            != Some(&AspectValue::UInt64(
                super::super::schema::version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION,
            ))
        {
            return Err(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance });
        }
        let branch_field = layout
            .instance
            .branch_occurrence
            .field_path()
            .fields()
            .first()
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        let authored_branch = fields
            .get(branch_field)
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        if authored_branch != &AspectValue::UInt64(branch_occurrence) {
            continue;
        }
        let field = layout
            .instance
            .program_revision
            .field_path()
            .fields()
            .first()
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        let revision = fields
            .get(field)
            .ok_or(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance })?;
        if !matches!(revision, AspectValue::String(_)) {
            return Err(WorkflowAdoptionInventoryDenial::UnreadableInstance { instance });
        }
        instances.push(instance);
    }
    instances.sort_unstable();
    instances.dedup();
    Ok(WorkflowAdoptionInventory {
        instances: instances.into_boxed_slice(),
        work_units,
    })
}
