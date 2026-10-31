use super::*;

pub(super) fn run_worker(
    owner: Weak<PhysicalMutationRuntimeOwner>,
    attempt: Arc<crate::physical_runtime::durability::mutation::PhysicalMutationAttempt>,
    prepared: PreparedPhysicalMutation,
) {
    let execution = catch_unwind(AssertUnwindSafe(|| {
        let Some(owner) = owner.upgrade() else {
            return PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    attempt.identity(),
                    attempt.idempotency_identity(),
                    attempt.fingerprint(),
                    PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                    0,
                ),
            );
        };
        let Some(director) = owner.director.upgrade() else {
            return PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    attempt.identity(),
                    attempt.idempotency_identity(),
                    attempt.fingerprint(),
                    PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                    0,
                ),
            );
        };
        owner
            .yieldpoint
            .pause(super::super::PhysicalMutationCheckpoint::BeforeEffectCutover);
        director.execute_managed_mutation(prepared, &attempt)
    }));
    let (terminal, panicked) = match execution {
        Ok(terminal) => (terminal, false),
        Err(_) => (
            attempt.retained_physical_fact().unwrap_or_else(|| {
                PhysicalMutationTerminalFact::Indeterminate(
                    IndeterminatePhysicalMutation::possible_effect(
                        attempt.identity(),
                        attempt.idempotency_identity(),
                        attempt.fingerprint(),
                        PhysicalMutationIndeterminateStage::WorkerPanicked,
                        0,
                    ),
                )
            }),
            true,
        ),
    };
    attempt.install_terminal(terminal.clone());
    let settlement = catch_unwind(AssertUnwindSafe(|| {
        let owner = owner.upgrade().ok_or(())?;
        owner
            .yieldpoint
            .pause(super::super::PhysicalMutationCheckpoint::BeforeTerminalFinalization);
        owner.record_terminal(&attempt, terminal.clone(), panicked);
        Ok::<(), ()>(())
    }));
    let failure = match settlement {
        Ok(Ok(())) => None,
        Ok(Err(())) => Some(crate::physical_runtime::AnchoredRecordStepSettlementFailure::WorkerUnavailable),
        Err(_) => Some(crate::physical_runtime::AnchoredRecordStepSettlementFailure::TerminalSettlementPanicked),
    };
    if let Some(cause) = failure {
        attempt.finish_anchored_step(&terminal, Some(cause));
        attempt.publish_terminal();
    }
}
