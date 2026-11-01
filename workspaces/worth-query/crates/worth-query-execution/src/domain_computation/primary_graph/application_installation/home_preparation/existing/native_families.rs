//! Inspect latest Native owner populations before any existing-Home service.
use super::super::*;
use worth_relational::facade::{identity::KindId, runtime::VisibilityProjectionView};
pub(super) fn readmit(
    native: &RelationalRuntime,
    layout: &super::super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .map_err(|cause| denied(format!("Home latest Native basis unavailable: {cause:?}")))?;
    let view = native
        .read_truth()
        .project_observation(&basis.observation())
        .map_err(|cause| denied(format!("Home latest Native view unavailable: {cause:?}")))?;
    for (family, kind) in [
        ("outbox", layout.provider_dispatch_outbox().entity_kind),
        ("inbound", layout.provider_inbound_completion().kind),
        (
            "aftermath",
            layout.provider_aftermath_causality().entity_kind,
        ),
    ] {
        absent(&view, execution, family, kind)?;
    }
    let (entities, relations) = layout.workflow().home_readmission_kinds();
    for kind in entities {
        absent(&view, execution, "workflow", kind)?;
    }
    for kind in relations {
        let rows = view
            .bounded_relations_of_kind(execution, kind, usize::MAX)
            .map_err(super::super::super::super::bootstrap::request_read_denial::read)?
            .map_err(|_| denied("Home workflow relation scan exceeded its bound"))?;
        if let Some(record) = rows.records().first() {
            return Err(denied(format!(
                "Home unsupported Native workflow relation kind {} record {:?}",
                kind.0, record.relation_id
            )));
        }
    }
    super::super::super::super::application_attempt::idempotency::home_readmission::readmit(
        native,
        &view,
        layout.provider_idempotency(),
        execution,
    )
}
fn absent(
    view: &VisibilityProjectionView,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    family: &str,
    kind: KindId,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let rows = view
        .bounded_entities_of_kind(execution, kind, usize::MAX)
        .map_err(super::super::super::super::bootstrap::request_read_denial::read)?
        .map_err(|_| denied("Home Native family scan exceeded its bound"))?;
    if let Some(record) = rows.records().first() {
        return Err(denied(format!(
            "Home unsupported Native {family} entity kind {} record {:?}",
            kind.0, record.entity_id
        )));
    }
    Ok(())
}
fn denied(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(super::super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected, subject)
}
