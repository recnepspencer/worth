use worth_foundational::{
    admit_authoritative_record_aspect_state, aspect_state_digest_preparation_basis, AspectValue,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

fn main() {
    let entry = {
        let contract = worth_foundational::AspectContract::scalar(
            worth_foundational::AspectKey::new("count").unwrap(),
            worth_foundational::AspectIdentity(1),
            worth_foundational::AspectContractRevision(1),
            ScalarAspectType::Int64,
        );
        let TransitionOutcome::Success(entry) =
            worth_foundational::validate_aspect_value(&contract, AspectValue::Int64(1).into())
        else {
            return;
        };
        entry
    };
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state([entry]) else {
        return;
    };

    let _basis = aspect_state_digest_preparation_basis(&state);
}
