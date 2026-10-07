//! Complete bounded entity revalidation; unsupported retained meaning refuses.
use crate::domain_computation::primary_graph::{
    schema_layout::WorthQueryPrimaryGraphLayout, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_declaration::facade::application_schema::ApplicationInvariantScopeTarget;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use worth_relational::facade::{identity::EntityId, runtime::VisibilityProjectionView};

pub(super) fn select<Schema: ApplicationSchema>(
    branch: &VisibilityProjectionView<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    maximum_work: usize,
    mut work: usize,
) -> Result<Vec<EntityId>, WorthQueryPrimaryGraphInstallationDenial> {
    // This first transition surface does not claim relation-rule or workflow
    // migration. Such meaning requires its own complete validation/dispositions.
    if installed.invariants().descriptors().any(|rule| {
        rule.applicability()
            .iter()
            .any(|scope| matches!(scope, ApplicationInvariantScopeTarget::Relation(_)))
    }) {
        return Err(
            WorthQueryPrimaryGraphInstallationDenial::checkpoint_recovery_rejected(
                "checkpoint transition does not support relation-scoped invariants",
            ),
        );
    }
    let workflow = layout.workflow();
    for kind in [
        workflow.lineage.entity_kind,
        workflow.definition.entity_kind,
        workflow.node.entity_kind,
        workflow.connection.entity_kind,
        workflow.instance.entity_kind,
        workflow.transition.entity_kind,
        workflow.proposal.entity_kind,
        workflow.proposal_coverage.entity_kind,
        workflow.assessment_evidence.entity_kind,
        workflow.approval.entity_kind,
        workflow.evidence_dependency.entity_kind,
    ] {
        let read = branch
            .bounded_entities_of_kind(kind, maximum_work.saturating_sub(work))
            .map_err(|_| {
                WorthQueryPrimaryGraphInstallationDenial::checkpoint_recovery_rejected(
                    "checkpoint transition workflow selection exceeded its bound",
                )
            })?;
        work = work.saturating_add(read.work_units());
        if !read.into_records().is_empty() {
            return Err(
                WorthQueryPrimaryGraphInstallationDenial::checkpoint_recovery_rejected(
                    "checkpoint transition requires workflow migration dispositions",
                ),
            );
        }
    }
    let mut entities = Vec::new();
    for kind in layout.application_entity_kinds() {
        let read = branch
            .bounded_entities_of_kind(kind, maximum_work.saturating_sub(work))
            .map_err(|_| {
                WorthQueryPrimaryGraphInstallationDenial::checkpoint_recovery_rejected(
                    "checkpoint transition entity selection exceeded its bound",
                )
            })?;
        work = work.saturating_add(read.work_units());
        entities.extend(
            read.into_records()
                .into_iter()
                .map(|record| record.entity_id),
        );
    }
    Ok(entities)
}
