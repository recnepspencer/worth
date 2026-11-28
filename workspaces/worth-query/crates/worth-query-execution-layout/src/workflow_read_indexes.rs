//! Native derived index identities used by bounded governed instance pages.
use crate::workflow_schema::WorthQueryWorkflowLayout;
use worth_relational::facade::indexes::{
    DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField,
};
#[derive(Debug, Clone, Copy)]
pub struct WorkflowReadIndexes {
    pub instances: DerivedIndexId,
    pub transitions: DerivedIndexId,
}
impl WorkflowReadIndexes {
    pub fn install(
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
            unique_entity_kind: None,
        })?;
        let transitions = install(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "worth-query-workflow.instance-transitions".to_owned(),
            branch_scoped: true,
            unique_entity_kind: None,
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
