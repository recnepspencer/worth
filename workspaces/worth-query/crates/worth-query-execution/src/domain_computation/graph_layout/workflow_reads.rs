//! Query read-index custody installed with this exact graph layout.
use super::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::graph_layout::workflow::WorthQueryWorkflowLayout;
use worth_relational::facade::indexes::{
    DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField,
};
impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation) fn workflow_reads(&self) -> Option<WorkflowReadIndexes> {
        self.workflow_reads
    }
    pub(in crate::domain_computation) fn set_workflow_reads(
        &mut self,
        indexes: WorkflowReadIndexes,
    ) {
        self.workflow_reads = Some(indexes);
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::domain_computation) struct WorkflowReadIndexes {
    pub(in crate::domain_computation) instances: DerivedIndexId,
    pub(in crate::domain_computation) transitions: DerivedIndexId,
}

impl WorkflowReadIndexes {
    pub(in crate::domain_computation) fn install(
        layout: &WorthQueryWorkflowLayout,
        mut install: impl FnMut(DerivedIndexDefinition) -> Result<DerivedIndexDefinition, String>,
    ) -> Result<Self, String> {
        let fields = [
            layout.instance.subject_partition.clone(),
            layout.instance.subject_slot.clone(),
            layout.instance.subject_generation.clone(),
            layout.instance.identity.clone(),
        ];
        let definition = install(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "worth-query-workflow.definition-instances".to_owned(),
            kind: DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: layout.instance_lineage_relation,
                parent_endpoint: RelatedEntityEndpoint::TargetParent,
                child_kind: layout.instance.entity_kind,
                ordering: fields
                    .into_iter()
                    .map(|field| {
                        RelatedEntityOrderingField::new(
                            field,
                            RelatedEntityOrderingDirection::Ascending,
                        )
                    })
                    .collect(),
            },
            branch_scoped: true,
        })?;
        let transitions = install(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "worth-query-workflow.instance-transitions".to_owned(),
            branch_scoped: true,
            kind: DerivedIndexKind::RelatedEntityOrdering {
                relation_kind: layout.instance_transition_relation,
                parent_endpoint: RelatedEntityEndpoint::SourceParent,
                child_kind: layout.transition.entity_kind,
                ordering: vec![RelatedEntityOrderingField::new(
                    layout.transition.occurrence.clone(),
                    RelatedEntityOrderingDirection::Ascending,
                )],
            },
        })?;
        Ok(Self {
            instances: definition.index_id,
            transitions: transitions.index_id,
        })
    }
}
