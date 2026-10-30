use worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission;
use worth_store::physical_runtime::{
    DataDispatchedPhysicalMutation, DataSettledPhysicalMutation, WalAppendedPhysicalMutation,
    WalDurablePhysicalMutation,
};

fn require_clone<T: Clone>() {}

fn main() {
    let _ = DataDispatchedPhysicalMutation {};
    let _ = DataSettledPhysicalMutation {};
    require_clone::<WalDurablePhysicalMutation>();
    require_clone::<DataDispatchedPhysicalMutation>();
    require_clone::<DataSettledPhysicalMutation>();
}

fn appended_wal_cannot_dispatch_data(
    submission: &CertificationPhysicalRecordSubmission,
    appended: WalAppendedPhysicalMutation,
) {
    let _ = submission.dispatch_wal_durable_data(appended);
}

fn wal_durable_cannot_skip_dispatch_and_settle(durable: WalDurablePhysicalMutation) {
    let _ = durable.settle_exact_effects();
}

fn raw_durable_and_effects_cannot_construct_dispatched(
    durable: WalDurablePhysicalMutation,
) -> DataDispatchedPhysicalMutation {
    DataDispatchedPhysicalMutation::new(durable, Vec::new())
}

fn dispatched_cannot_construct_settled(
    dispatched: DataDispatchedPhysicalMutation,
) -> DataSettledPhysicalMutation {
    DataSettledPhysicalMutation::new(dispatched)
}
