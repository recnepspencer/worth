//! Workflow-owned lookup registration.

use super::WorthQueryWorkflowLayout;
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};

pub(in crate::domain_computation::primary_graph) fn register_indexes(
    layout: &mut WorthQueryWorkflowLayout,
    mut install: impl FnMut(DerivedIndexDefinition) -> Result<DerivedIndexDefinition, String>,
) -> Result<(), String> {
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.lineage-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.lineage.identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.lineage.identity_index_id = installed.index_id;
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.definition-content-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.definition.content_identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.definition.content_identity_index_id = installed.index_id;
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.instance-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.instance.identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.instance.identity_index_id = installed.index_id;
    let installed = install(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "worth-query-workflow.proposal-identity".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: layout.proposal.identity.clone(),
        },
        branch_scoped: true,
    })?;
    layout.proposal.identity_index_id = installed.index_id;
    Ok(())
}
