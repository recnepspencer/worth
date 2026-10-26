//! Candidate lookup work is bounded by its declared keys, not their product.
use super::*;
use crate::execution::{execute_read_only_packets, ReadOnlyPacket};
use crate::transactions::data::{
    AspectFieldPatch, BulkEntityCreateIntent, CreateIntent, MergedCommitPlan, MutationIntent,
    TransactionId,
};
use crate::validation::engine::{
    budget::LeasedInvariantBudget, InvariantObservation, InvariantRuntimeView,
};
use std::num::NonZeroUsize;
use std::sync::{Arc, OnceLock};
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

#[test]
fn created_endpoint_lookups_fit_the_declared_subquadratic_work() {
    const KEYS: usize = 128;
    // Count, fill, sorting comparisons and lookup comparisons: at most four
    // visits per key per binary-tree level, including the two construction passes.
    let levels = u64::from(usize::BITS - KEYS.leading_zeros());
    let allowance = 4 * KEYS as u64 * levels;
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::schema::data::RelationalSchemaRegistry::new())
        .build();
    let keys: Vec<_> = (0..KEYS)
        .rev()
        .map(|i| crate::symbols::data::ClientKey::raw(format!("key-{i}")))
        .collect();
    let plan = MergedCommitPlan {
        transaction_id: TransactionId(1),
        merged_intents: vec![MutationIntent::Create(CreateIntent::BulkEntities(
            BulkEntityCreateIntent {
                partition_id: crate::identity::data::PartitionId::main(),
                kind_id: crate::identity::data::KindId(7),
                client_keys: keys.clone(),
                field_patches: vec![AspectFieldPatch::default(); KEYS],
            },
        ))],
    };
    let mut scope = crate::validation::engine::request::PreparedRelationIntegrityScope::default();
    for key in &keys {
        let reference = EntityReference::Created(CreatedEntityRef {
            partition_id: crate::identity::data::PartitionId::main(),
            kind_id: crate::identity::data::KindId(7),
            client_key: key.clone(),
        });
        scope.increment_counts(reference.clone(), reference);
    }
    let scopes = crate::validation::engine::PreparedRelationIntegrityScopes::new(
        [(crate::identity::data::KindId(8), scope)].into(),
    )
    .with_created_key_count(KEYS);
    for (posture, work, scratch) in [ExecutionPosture::Serial, ExecutionPosture::Automatic]
        .into_iter()
        .flat_map(|posture| {
            [
                (posture, allowance, KEYS as u64 * 128),
                // No scratch is available. The original scan still fits its exact
                // declared packet entry plus 2+...+(N+1) work; an index must not refuse it.
                (posture, 1 + KEYS as u64 * (KEYS as u64 + 3) / 2, 0),
            ]
        })
    {
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, None, Some(work)),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        let result = execute_read_only_packets(
            vec![ReadOnlyPacket {
                identity: PartitionIdentity::new(1),
                value: (),
                input_bytes: 0,
                memory: crate::execution::PacketMemory::Declared { scratch, result: 0 },
            }],
            &lease,
            |_, packet_context| {
                let budget = LeasedInvariantBudget::new(packet_context, lease.status());
                let context = InvariantExecutionContext::new(
                    &InvariantRuntimeView::from_runtime(&runtime),
                    InvariantObservation::committed(runtime.storage_access().current_edition()),
                    runtime.current_version_id(),
                    runtime.current_version_id(),
                    Some(&plan),
                    Some(scopes.clone()),
                    Arc::new(OnceLock::new()),
                )
                .with_budget(&budget)
                .with_created_endpoint_scope(crate::identity::data::KindId(8));
                for key in &keys {
                    let reference = EntityReference::Created(CreatedEntityRef {
                        partition_id: crate::identity::data::PartitionId::main(),
                        kind_id: crate::identity::data::KindId(7),
                        client_key: key.clone(),
                    });
                    let actual = entity_reference_kind(
                        &context,
                        crate::validation::data::InvariantClass::CommitBoundary,
                        &reference,
                    )
                    .unwrap();
                    if let Some(failure) = budget.take_failure() {
                        return Err(failure);
                    }
                    assert_eq!(actual, Some(crate::identity::data::KindId(7)));
                }
                Ok(())
            },
            |_| 0,
        );
        assert_eq!(
            result.expect("all declared endpoints fit the nonquadratic request"),
            vec![()]
        );
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
