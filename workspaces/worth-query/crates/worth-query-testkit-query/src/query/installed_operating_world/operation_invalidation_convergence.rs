pub use worth_query::facade::{certification, domain, runtime};

pub use super::installed_operation_fixture::{
    consume_empty_invalidation_epoch as consume_empty_epoch, lineage_invalidation_workspace,
    settle_native, InvalidationLease as Lease,
};
pub use super::operation_lineage::{bind, intent};

pub struct ObservedInvalidation {
    pub semantic: domain::WorthQueryConsumerInvalidationSemanticProjection,
    pub admitted_bytes: [u8; 32],
    pub compatibility: domain::WorthQueryInvalidationCompatibilityOutcome,
    pub foundational: domain::WorthQueryFoundationalInvalidationProjection,
    pub maintenance_ordinal: u64,
    pub counters: domain::WorthQueryConsumerInvalidationCounters,
    pub epoch_counters: domain::WorthQueryConsumerInvalidationEpochCounters,
}

pub fn observe_delivery(
    lease: &Lease,
    delivery: domain::WorthQuerySharedProjectionDelivery,
    workspace: &runtime::WorthQueryWorkspace,
) -> ObservedInvalidation {
    let delta = lease.consumer_invalidation_delta(delivery).unwrap();
    let semantic = delta.semantic_projection();
    let foundational = delta.foundational_projection();
    let maintenance_ordinal = delta.maintenance_ordinal();
    let counters = delta.counters();
    let epoch_counters = delta.epoch_counters();
    let admitted = lease
        .admit_consumer_invalidation_delta(delta, workspace)
        .unwrap_or_else(|_| panic!("current replay-convergence delta did not readmit"));
    let admitted = admitted
        .admitted_semantic_projection(workspace)
        .expect("current admitted delta retains semantic continuity");
    ObservedInvalidation {
        semantic,
        admitted_bytes: *admitted.canonical_bytes(),
        compatibility: admitted.compatibility(),
        foundational,
        maintenance_ordinal,
        counters,
        epoch_counters,
    }
}
