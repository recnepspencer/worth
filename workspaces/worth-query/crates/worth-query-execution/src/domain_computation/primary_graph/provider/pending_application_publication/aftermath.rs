//! Seals a co-committed aftermath fact into its commit's evidence at publication.
//!
//! Evidence outlives its commit's history, so the exact committed relation is
//! read once, at the published basis, and replay answers from the sealed copy
//! instead of a Relational read that history retirement may since have made
//! unreachable.

use super::stops::{failure, snapshot_capacity_failure};
use super::WorthQueryPendingApplicationPublication;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial;
use crate::domain_computation::WorthQueryProviderSessionFailure;

pub(super) fn seal(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), WorthQueryProviderSessionFailure> {
    let Some(relation) = pending
        .attempt
        .as_ref()
        .and_then(|attempt| attempt.aftermath_causality())
        .cloned()
    else {
        return Ok(());
    };
    let application = pending
        .application
        .as_ref()
        .expect("pending application retains its evidence until cutover");
    if application.aftermath_causality().is_some() {
        return Ok(());
    }
    let committed = provider
        .resolve_aftermath_causality_in_snapshot(
            execution,
            runtime,
            &pending.next_basis,
            after,
            &relation,
            Some(pending.outcome_identity),
        )
        .map_err(sealing_denial)?
        .filter(|committed| committed.child() == application.commit_reference())
        .ok_or_else(|| failure("published commit does not carry its aftermath causality"))?;
    pending
        .application
        .as_mut()
        .expect("pending application retains its evidence until cutover")
        .seal_aftermath_causality(committed);
    Ok(())
}

fn sealing_denial(
    denial: WorthQueryAftermathCausalityReadDenial,
) -> WorthQueryProviderSessionFailure {
    match denial {
            WorthQueryAftermathCausalityReadDenial::ExecutionDenied(stop) => WorthQueryProviderSessionFailure::checked_read_stopped(stop,
                crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
                "request stopped during aftermath sealing")
                .with_recovery_posture(crate::domain_computation::WorthQueryProviderSessionRecoveryPosture::RecoveryRequired),
            WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted { .. } => {
                snapshot_capacity_failure("aftermath causality could not be read at publication")
            }
            _ => failure("aftermath causality could not be read at publication"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_aftermath_stop_keeps_recovery_custody_by_value() {
        use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
        use crate::domain_computation::{
            WorthQueryProviderSessionControlStopKind as Control,
            WorthQueryProviderSessionDenialKind as Kind,
            WorthQueryProviderSessionRecoveryPosture as Posture,
        };
        for stop in [
            Err(Control::Cancelled),
            Ok(Kind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: None,
                policy_ancestor: None,
            }),
        ] {
            let failure = sealing_denial(WorthQueryAftermathCausalityReadDenial::ExecutionDenied(
                stop,
            ));
            assert_eq!(failure.checked_read_stop(), Some(stop));
            assert_eq!(failure.recovery_posture(), Posture::RecoveryRequired);
            assert_eq!(
                failure.stage(),
                crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit
            );
        }
    }
}
