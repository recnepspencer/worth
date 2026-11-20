//! A recovered original branch creation carries no Query key: the fate its
//! owners recorded is its only admissible decision. Deciding it before the
//! branch inventory attaches a continued original exactly as a branch whose
//! selection was durable before the kill.
use super::*;

pub(super) fn decide_recovered_creations(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    inspection: &RuntimeWorldInspectionPort,
    branch: &RuntimeWorldBranchPort,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let execution = phase.execution_request();
    let mut after = None;
    loop {
        let page = inspection
            .recovery_page(
                after.as_ref(),
                std::num::NonZeroUsize::new(32).expect("nonzero page"),
            )
            .map_err(|cause| home_world_denial(format!("Home recovery page refused: {cause:?}")))?;
        for row in page.rows() {
            match row.recovered_original() {
                Some(RuntimeWorldRecoveredOriginal::BranchCreation) => {}
                Some(RuntimeWorldRecoveredOriginal::Publication) | None => continue,
            }
            let cancellation = RuntimeWorldCancellationSource::new();
            match branch.resume_creation(&execution, row.handle(), &cancellation.token()) {
                Ok(
                    RuntimeWorldBranchCreationOutcome::Performed(_)
                    | RuntimeWorldBranchCreationOutcome::ReleasedOriginal(_),
                ) => {}
                other => {
                    return Err(home_world_denial(format!(
                        "recovered original branch creation was not decided: {other:?}"
                    )))
                }
            }
        }
        after = page.next_after().cloned();
        if after.is_none() {
            return Ok(());
        }
    }
}
