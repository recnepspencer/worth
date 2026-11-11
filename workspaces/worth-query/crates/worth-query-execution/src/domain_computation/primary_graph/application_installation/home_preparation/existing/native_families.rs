//! Original-key integrity before Home service; outboxes and completions readmit at D8.
use super::super::*;
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
    // Completion rows join their actual World originals and original outboxes
    // in readmit_inbound_terminal_sources before the service witness escapes.
    super::super::super::super::application_attempt::idempotency::home_readmission::readmit(
        native,
        &view,
        layout.provider_idempotency(),
        execution,
    )
}
fn denied(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(super::super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected, subject)
}
