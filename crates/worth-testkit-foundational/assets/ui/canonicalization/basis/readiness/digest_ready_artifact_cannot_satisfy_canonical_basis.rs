use worth_foundational::{
    prepare_aspect_contract_for_digest, AspectContract, AspectContractRevision, AspectIdentity,
    AspectKey, CanonicalBasisReadyArtifact, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

fn requires_canonical_basis(_: CanonicalBasisReadyArtifact) {}

fn main() {
    let contract = AspectContract::scalar(
        AspectKey::new("task.summary").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let TransitionOutcome::Success(ready) = prepare_aspect_contract_for_digest(contract) else {
        return;
    };

    requires_canonical_basis(ready);
}
