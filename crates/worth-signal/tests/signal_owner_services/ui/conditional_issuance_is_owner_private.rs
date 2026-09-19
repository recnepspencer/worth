use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalConditionalExecutionPort};
use worth_signal::facade::SignalAspectLoweringOwner;

fn forge(basis: &AdmittedSignalBranchBasis, claimant: &SignalAspectLoweringOwner) {
    let source_owner = worth_proof::ConditionalSourceObservationOwner::fresh();
    let _ = SignalConditionalExecutionPort::<(), (), ()>::issue(
        Default::default(),
        basis,
        claimant,
        &source_owner.authority(),
    );
}

fn main() {}
