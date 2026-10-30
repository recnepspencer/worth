use worth_foundational::{
    compatibility, validate_aspect_value, AspectContract, AspectContractRevision, AspectIdentity,
    AspectKey, AspectValue, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

fn main() {
    let contract = AspectContract::scalar(
        AspectKey::new("retry.count").expect("valid aspect key"),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::Int64,
    );
    let TransitionOutcome::Success(validated) =
        validate_aspect_value(&contract, AspectValue::Int64(3).into())
    else {
        panic!("expected validated artifact");
    };

    let _ = compatibility().json().lower_state([validated]);
}
