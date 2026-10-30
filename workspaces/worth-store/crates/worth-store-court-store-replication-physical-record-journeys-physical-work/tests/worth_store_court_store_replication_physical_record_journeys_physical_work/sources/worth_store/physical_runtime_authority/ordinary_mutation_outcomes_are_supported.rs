use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess,
};

fn consume_preparation(outcome: PhysicalMutationPreparationOutcome) {
    match outcome.into_raw() {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            let handle = prepared.start();
            consume_terminal(handle.wait());
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(completed)) => {
            let acknowledgment = completed.into_acknowledgment();
            let _ = acknowledgment.executed_boundary_evidence();
            let _ = acknowledgment.performance_evidence();
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(fate)) => {
            let _ = fate.diagnostic_evidence();
        }
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(fate)) => {
            let _ = fate.diagnostic_evidence();
        }
        TransitionOutcome::Denied(denial) => {
            let _ = denial;
        }
        TransitionOutcome::Deferred(deferred) => {
            let _ = deferred;
        }
        TransitionOutcome::Stale(stale) => {
            let _ = stale;
        }
        TransitionOutcome::RebindRequired(rebind) => {
            let _ = rebind;
        }
        TransitionOutcome::Failed(failure) => {
            let _ = failure;
        }
    }
}

fn consume_terminal(outcome: PhysicalMutationOutcome) {
    match outcome {
        PhysicalMutationOutcome::Completed(completed) => {
            let _ = completed.into_acknowledgment();
        }
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            let _ = fate.diagnostic_evidence();
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            let _ = fate.diagnostic_evidence();
        }
    }
}

fn main() {}
