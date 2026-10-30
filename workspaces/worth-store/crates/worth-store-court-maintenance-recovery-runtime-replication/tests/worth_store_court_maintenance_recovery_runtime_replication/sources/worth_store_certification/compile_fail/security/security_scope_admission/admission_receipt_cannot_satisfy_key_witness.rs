use worth_store_security::{
    StoreCurrentKeyScopeWitness, StoreSecurityScopeAdmissionReceipt,
};

fn require_key_scope_witness(_: StoreCurrentKeyScopeWitness) {}

fn main() {
    let receipt: StoreSecurityScopeAdmissionReceipt = unimplemented!();
    require_key_scope_witness(receipt);
}
