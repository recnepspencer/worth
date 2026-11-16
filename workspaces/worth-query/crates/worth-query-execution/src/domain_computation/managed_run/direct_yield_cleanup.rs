mod inspection;

use super::WorthQueryManagedRelationalObservation;
use worth_runtime_bridge::facade::BridgeYieldedExecutionBasis;

pub use inspection::{WorthQueryDirectYieldCleanupInspection, WorthQueryDirectYieldCleanupReceipt};

use self::inspection::WorthQueryCompletedDirectYieldCleanup;
use super::{
    run_affinity::WorthQueryDirectRunTerminalAffinity, WorthQueryDirectYieldRecoveryRequired,
    WorthQueryManagedProviderWorkEvidence, WorthQueryManagedRunCleanupDisposition,
    WorthQueryManagedRunCounters, WorthQueryYieldRecoveryResourceEvidence,
    WorthQueryYieldTransitionCounters, WorthQueryYieldedDirectRun,
};

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
    provider_cleanup: super::provider_work::WorthQueryManagedProviderCleanupAuthority,
    yield_counters: WorthQueryYieldTransitionCounters,
}

pub(super) fn cleanup_yielded_direct(
    yielded: WorthQueryYieldedDirectRun,
) -> WorthQueryDirectYieldCleanupOutcome {
    let permit = WorthQueryDirectYieldCleanupPermit::mint();
    let (mut affinity, relational_basis, bridge, execution, run_counters, yield_counters) =
        yielded.owner_into_cleanup_parts(&permit);
    let memory = execution.memory.clone();
    let checkpoint_release = execution.release();
    affinity.provider_work_mut().retain_provider_memory(memory);
    let (affinity, provider_work, provider_cleanup) = affinity.into_terminal_parts();
    let (logical_run_identity, attempt_identity) = affinity.terminal_descriptions();
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
            provider_cleanup,
            yield_counters,
        },
        Some(checkpoint_release),
        None,
        recovery_required,
    );
    if receipt.inspection().disposition()
        == WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    {
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
        provider_cleanup,
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
            provider_cleanup,
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
        mut provider_work,
        mut provider_cleanup,
        yield_counters,
    } = association;
    provider_work.reconcile_provider_retained_bytes(provider_cleanup.reconcile_provider_memory());
    let recovery_required = recovery_required || provider_work.provider_retained_bytes() != 0;
    let custody = affinity.custody();
    let checkpoint = checkpoint_release
        .as_ref()
        .map(|release| release.disposition())
        .or_else(|| {
            let evidence = recovery_evidence.as_ref()?;
            evidence
                .checkpoint_release()
                .map(|release| release.disposition())
                .or_else(|| {
                    let failure = evidence.provider_checkpoint_failure()?;
                    failure
                        .checkpoint_release()
                        .map(|release| release.disposition())
                        .or_else(|| {
                            failure
                                .checkpoint_retention_failure()
                                .map(|failure| failure.release_disposition())
                        })
                })
        });
    let receipt = WorthQueryDirectYieldCleanupReceipt::from_completed(
        WorthQueryCompletedDirectYieldCleanup {
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
        },
    );
    if let Some(custody) = custody {
        let inspection = receipt.inspection();
        let mut resources = crate::domain_computation::primary_graph::query_obligations::QueryManagedRunResources::observe(
            inspection.resources_released(), inspection.provider_work());
        resources.checkpoint = checkpoint;
        custody.record(crate::domain_computation::primary_graph::query_obligations::QueryManagedRunPosture::YieldedCleanup {
            disposition: inspection.disposition(), resources,
        }, inspection.yielded_attempt_identity());
    }
    receipt
}
