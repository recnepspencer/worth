use worth_store::physical_runtime::{StoreRecoveryBindingFreshnessSample, StoreRecoveryWalMember};

fn clone_sample(
    sample: &StoreRecoveryBindingFreshnessSample,
) -> StoreRecoveryBindingFreshnessSample {
    <StoreRecoveryBindingFreshnessSample as Clone>::clone(sample)
}

fn clone_member(member: &StoreRecoveryWalMember) -> StoreRecoveryWalMember {
    <StoreRecoveryWalMember as Clone>::clone(member)
}

fn main() {}
