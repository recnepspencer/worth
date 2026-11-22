//! Reconstructed correspondence keeps its actual payer through every shared clone.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::with_bootstrap_advancement,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn reconstructed_roles_retain_their_payer_until_the_last_clone_and_refuse_without_memory() {
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(1 << 20), Some(1_000_000)),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let worth_execution::ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("bootstrap must carry its actual lease")
        };
        let roles = [WorthQueryCheckpointOutputRole {
            role: "subject".into(),
            posture: super::super::WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "entity".into(),
            entity: EntityId::new(PartitionId::main(), 1, 1),
        }];
        let reconstruct = || WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            TypeId::of::<()>(), TypeId::of::<()>(), ["companion"].into_iter(),
            &roles, |_| Some(TypeId::of::<()>()), &execution);
        let before = lease.charged_memory_bytes();
        let original = std::sync::Arc::new(reconstruct().unwrap());
        let charged = lease.charged_memory_bytes();
        assert!(charged > before);
        let registry = std::sync::Arc::clone(&original);
        let lineage = std::sync::Arc::clone(&registry);
        // Even a value clone shares the same immutable heaps and linear payer.
        let value_clone = original.as_ref().clone();
        assert!(std::sync::Arc::ptr_eq(&original.storage, &value_clone.storage));
        drop(original);
        drop(registry);
        drop(lineage);
        assert_eq!(lease.charged_memory_bytes(), charged);
        assert_eq!(value_clone.publication_entity_for_role("subject"), Some(roles[0].entity));
        drop(value_clone);
        assert_eq!(lease.charged_memory_bytes(), before);

        let held = lease.reserve_memory(lease.available_memory_bytes().expect("the proof declares a finite memory limit")).unwrap();
        let denial = reconstruct().unwrap_err().into_installation_denial();
        use crate::domain_computation::{WorthQueryProviderSessionDenialKind as Provider,
            primary_graph::application_contribution::{WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level}};
        assert!(matches!(denial.kind(), Kind::ExecutionDenied {
            kind: Provider::ExecutionResource { denial: Resource::MemoryLimit { requested, admitted, level: Level::Policy }, .. }
        } if requested > admitted));
        assert_eq!(roles[0].role, "subject");
        assert_eq!(roles[0].entity_name, "entity");
        drop(held);
        let retry = reconstruct().unwrap();
        assert_eq!(retry.publication_entity_for_role("subject"), Some(roles[0].entity));
        drop(retry);
        assert_eq!(lease.charged_memory_bytes(), before);
    }).unwrap();
}
