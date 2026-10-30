pub use worth_proof::TransitionOutcome;
pub use {
    worth_foundational::validate_aspect_value, worth_foundational::AspectContract,
    worth_foundational::AspectContractRevision, worth_foundational::AspectIdentity,
    worth_foundational::AspectKey, worth_foundational::AspectValue,
    worth_foundational::ContractValidatedAspectArtifact, worth_foundational::FieldKey,
    worth_foundational::ScalarAspectType,
};

pub fn key(name: &str) -> AspectKey {
    worth_foundational::AspectKey::new(name).expect("valid aspect key")
}

pub fn field(name: &str) -> FieldKey {
    worth_foundational::FieldKey::new(name).expect("valid field key")
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
    worth_foundational::AspectContract::scalar(
        key(name),
        identity(aspect_identity),
        revision(1),
        scalar,
    )
}

pub fn validated_scalar(
    name: &str,
    aspect_identity: u64,
    scalar: ScalarAspectType,
    value: AspectValue,
) -> ContractValidatedAspectArtifact {
    let contract = scalar_contract(name, aspect_identity, scalar);
    let worth_proof::TransitionOutcome::Success(artifact) =
        validate_aspect_value(&contract, value.into())
    else {
        panic!("expected validated scalar artifact");
    };
    artifact
}
