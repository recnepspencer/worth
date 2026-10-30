fn main() {
    let contract = worth_foundational::AspectContract::scalar(
        worth_foundational::AspectKey::new("counter").unwrap(),
        worth_foundational::AspectIdentity(1),
        worth_foundational::AspectContractRevision(1),
        worth_foundational::ScalarAspectType::Int64,
    );

    worth_foundational::aspect_contract_digest_preparation_basis(&contract);
}
