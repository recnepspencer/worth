use worth_foundational::{
    AspectContractRevision, AspectKey, AspectValue, ContractValidatedAspectValue,
};

fn main() {
    let _worthd = ContractValidatedAspectValue::scalar(
        AspectKey::new("count").unwrap(),
        AspectValue::Int64(1),
        AspectContractRevision(1),
    );
}
