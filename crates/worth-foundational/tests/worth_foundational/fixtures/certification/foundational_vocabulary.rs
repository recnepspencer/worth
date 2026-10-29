use worth_foundational::{
    validate_aspect_value, AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
    AspectValue, ContractValidatedAspectArtifact, FieldKey, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

pub fn key(name: &str) -> AspectKey {
    AspectKey::new(name).expect("valid aspect key")
}

pub fn field(name: &str) -> FieldKey {
    FieldKey::new(name).expect("valid field key")
}

pub fn revision(value: u64) -> AspectContractRevision {
    AspectContractRevision(value)
}

pub fn identity(value: u64) -> AspectIdentity {
    AspectIdentity(value)
}

pub fn scalar_contract(
    name: &str,
    aspect_identity: u64,
    scalar: ScalarAspectType,
) -> AspectContract {
    AspectContract::scalar(key(name), identity(aspect_identity), revision(1), scalar)
}

pub fn validated_scalar(
    name: &str,
    aspect_identity: u64,
    scalar: ScalarAspectType,
    value: AspectValue,
) -> ContractValidatedAspectArtifact {
    let contract = scalar_contract(name, aspect_identity, scalar);
    let TransitionOutcome::Success(artifact) = validate_aspect_value(&contract, value.into())
    else {
        panic!("expected validated scalar artifact");
    };
    artifact
}
