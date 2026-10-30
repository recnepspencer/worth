use worth_foundational::{AspectContractRevision, AspectKey, ContractValidatedAspectValue};

fn main() {
    let _worthd = ContractValidatedAspectValue {
        key: AspectKey::new("count").unwrap(),
        contract_revision: AspectContractRevision(1),
    };
}
