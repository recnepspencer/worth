//! Native derived index identities used by bounded governed instance pages.
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;
use worth_relational::facade::indexes::{
    DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind, RelatedEntityEndpoint,
    RelatedEntityOrderingDirection, RelatedEntityOrderingField,
};
#[derive(Debug, Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct WorkflowReadIndexes {
    pub(in crate::domain_computation::primary_graph) instances: DerivedIndexId,
}
impl WorkflowReadIndexes {
    pub(in crate::domain_computation::primary_graph) fn install(
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
        Ok(Self {
            instances: definition.index_id,
        })
    }
}
