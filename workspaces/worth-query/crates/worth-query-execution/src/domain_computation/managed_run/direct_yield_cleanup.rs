mod inspection;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

pub use inspection::{WorthQueryDirectYieldCleanupInspection, WorthQueryDirectYieldCleanupReceipt};

use self::inspection::WorthQueryCompletedDirectYieldCleanup;
use {super::run_affinity::WorthQueryDirectRunTerminalAffinity, super::WorthQueryDirectYieldRecoveryRequired, crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence, crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition, crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters, super::WorthQueryYieldRecoveryResourceEvidence, super::WorthQueryYieldTransitionCounters, super::WorthQueryYieldedDirectRun};

pub(super) struct WorthQueryDirectYieldCleanupPermit {
    _owner: (),
}

impl WorthQueryDirectYieldCleanupPermit {
    fn mint() -> Self {
        Self { _owner: () }
    }
}

#[must_use = "direct yielded-run cleanup outcomes must be resolved"]
pub enum WorthQueryDirectYieldCleanupOutcome {
    Complete(WorthQueryDirectYieldCleanupReceipt),
    RecoveryRequired(WorthQueryDirectYieldCleanupReceipt),
}

struct WorthQueryDirectYieldCleanupAssociation {
    logical_run_identity: std::sync::Arc<str>,
    attempt_identity: std::sync::Arc<str>,
    affinity: WorthQueryDirectRunTerminalAffinity,
    relational_basis: WorthQueryManagedRelationalObservation,
    bridge: BridgeYieldedExecutionBasis,
    run_counters: WorthQueryManagedRunCounters,
    provider_work: WorthQueryManagedProviderWorkEvidence,
    yield_counters: WorthQueryYieldTransitionCounters,
}

pub(super) fn cleanup_yielded_direct(
    yielded: WorthQueryYieldedDirectRun,
) -> WorthQueryDirectYieldCleanupOutcome {
    let permit = WorthQueryDirectYieldCleanupPermit::mint();
    let (affinity, relational_basis, bridge, execution, run_counters, yield_counters) =
        yielded.owner_into_cleanup_parts(&permit);
    let (affinity, provider_work, _) = affinity.into_terminal_parts();
    let (logical_run_identity, attempt_identity) = affinity.terminal_descriptions();
    let checkpoint_release = execution.release();
    let recovery_required = checkpoint_release.disposition().recovery_required();
    let receipt = complete_cleanup(
        WorthQueryDirectYieldCleanupAssociation {
            logical_run_identity,
            attempt_identity,
            affinity,
            relational_basis,
            bridge,
            run_counters,
            provider_work,
            yield_counters,
        },
        Some(checkpoint_release),
        None,
        recovery_required,
    );
    if recovery_required {
        WorthQueryDirectYieldCleanupOutcome::RecoveryRequired(receipt)
    } else {
        WorthQueryDirectYieldCleanupOutcome::Complete(receipt)
    }
}

pub(super) fn cleanup_terminalized(
    recovery: WorthQueryDirectYieldRecoveryRequired,
) -> Result<WorthQueryDirectYieldCleanupReceipt, WorthQueryDirectYieldRecoveryRequired> {
    let permit = WorthQueryDirectYieldCleanupPermit::mint();
    let (
        logical_run_identity,
        attempt_identity,
        affinity,
        relational_basis,
        bridge,
        run_counters,
        provider_work,
        yield_counters,
        recovery_evidence,
    ) = recovery.owner_into_terminal_cleanup_parts(&permit)?;
    Ok(complete_cleanup(
        WorthQueryDirectYieldCleanupAssociation {
            logical_run_identity,
            attempt_identity,
            affinity,
            relational_basis,
            bridge,
            run_counters,
            provider_work,
            yield_counters,
        },
        None,
        Some(recovery_evidence),
        true,
    ))
}

fn complete_cleanup(
    association: WorthQueryDirectYieldCleanupAssociation,
    checkpoint_release: Option<
        crate::domain_computation::WorthQueryProviderCheckpointReleaseEvidence,
    >,
    recovery_evidence: Option<WorthQueryYieldRecoveryResourceEvidence>,
    recovery_required: bool,
) -> WorthQueryDirectYieldCleanupReceipt {
    let WorthQueryDirectYieldCleanupAssociation {
        logical_run_identity,
        attempt_identity,
        affinity,
        relational_basis,
        bridge,
        run_counters,
        provider_work,
        yield_counters,
    } = association;
    WorthQueryDirectYieldCleanupReceipt::from_completed(WorthQueryCompletedDirectYieldCleanup {
        logical_run_identity,
        attempt_identity,
        disposition: if recovery_required {
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        } else {
            WorthQueryManagedRunCleanupDisposition::CleanupComplete
        },
        checkpoint_release,
        recovery_evidence,
        bridge: bridge.release(),
        relational: relational_basis.release(),
        attempt: affinity.release(),
        run_counters,
        provider_work,
        yield_counters,
    })
}
