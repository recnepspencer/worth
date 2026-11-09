//! Non-workflow unsupported families and original-key integrity before Home service.
//! Workflow DATA joins its actual World branch history at the later custody/source gate.
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
