use worth_store::physical_runtime::{
    PhysicalDurabilityRecoveryHandoff, PhysicalRecoveryOperationFact,
};

fn promote(fact: PhysicalRecoveryOperationFact) -> PhysicalDurabilityRecoveryHandoff {
    fact.into()
}

fn main() {}
