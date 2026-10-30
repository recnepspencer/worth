use worth_store::physical_runtime::{
    PhysicalWalBarrierSettlement, WalAppendedPhysicalMutation, WalDurablePhysicalMutation,
};

fn require_clone<T: Clone>() {}

fn main() {
    let _ = PhysicalWalBarrierSettlement {};
    let _ = WalDurablePhysicalMutation {};
    require_clone::<WalDurablePhysicalMutation>();
}

fn raw_append_and_observed_settlement_are_insufficient(
    appended: WalAppendedPhysicalMutation,
    settlement: PhysicalWalBarrierSettlement,
) -> WalDurablePhysicalMutation {
    WalDurablePhysicalMutation::new(appended, settlement)
}
