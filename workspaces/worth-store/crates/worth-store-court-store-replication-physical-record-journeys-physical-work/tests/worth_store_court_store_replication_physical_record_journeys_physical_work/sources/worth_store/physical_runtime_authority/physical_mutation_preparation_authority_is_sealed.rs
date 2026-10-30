use worth_store::physical_runtime::{
    PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyLease, PhysicalMutationIdentity,
    PhysicalMutationRequestFingerprint, PreparedPhysicalMutation,
};

fn require_clone<T: Clone>() {}

fn main() {
    let _ = PhysicalMutationIdempotencyLease {};
    let _ = PhysicalMutationIdempotencyKey {};
    let _ = PhysicalMutationRequestFingerprint {};
    let _ = PhysicalMutationIdentity {};
    let _ = PreparedPhysicalMutation {};

    require_clone::<PreparedPhysicalMutation>();
}

fn allocation_cannot_change_equivalence(fingerprint: PhysicalMutationRequestFingerprint) {
    let _ = fingerprint.with_wal_allocation(1_u64);
}
