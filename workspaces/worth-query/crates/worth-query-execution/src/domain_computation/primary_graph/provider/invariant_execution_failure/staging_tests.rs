//! Actual Relational staging refusals retain their owner evidence through Query.
use super::map_transaction_staging_failure;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind as Kind,
    WorthQueryInvariantExecutionFailurePosture as Posture,
};
use worth_relational::facade::{
    config::RelationalRuntimeProfile,
    identity::{KindId, PartitionId},
    mvcc::{RelationalTransactionIntent, RelationalTransactionStagingDenial as Staging},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    symbols::ClientKey,
    transactions::{AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

#[test]
fn invariant_staging_capacity_cause_survives_application_commit_mapping() {
    for (maximum_bytes, maximum_loci) in [(0, 8), (1_048_576, 0)] {
        let runtime = runtime_with_bounds(maximum_bytes, maximum_loci);
        let identity = runtime.main_branch_identity();
        let basis = runtime.admit_branch_basis(&identity).unwrap();
        let before = runtime
            .branch_reference_state(identity.branch_id())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        // Staging refuses before schema validation, so kind/schema validity is
        // immaterial to this real footprint admission probe.
        let batch = WorkerIntentBatch::new("invariant-staging-capacity").push(
            MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId::new(1),
                client_key: ClientKey::raw("unstaged-record"),
                fields: AspectFieldPatch::default(),
            })),
        );
        let owner_denial = transaction.push_batch(batch).unwrap_err();
        let expected = match owner_denial {
            Staging::OverlayCapacityExhausted {
                maximum_bytes,
                required_bytes,
            } => {
                assert_eq!(maximum_bytes, 0);
                assert!(required_bytes > 0);
                Kind::TransactionOverlayCapacityExhausted {
                    maximum_bytes,
                    required_bytes,
                }
            }
            Staging::FootprintCapacityExhausted {
                maximum_loci,
                required_loci,
            } => {
                assert_eq!((maximum_loci, required_loci), (0, 1));
                Kind::TransactionFootprintCapacityExhausted {
                    maximum_loci,
                    required_loci,
                }
            }
            other => panic!("the real staging capacity must refuse first: {other:?}"),
        };
        assert_eq!(transaction.footprint().writes().len(), 0);
        let failure = map_transaction_staging_failure(owner_denial);
        let denial = WorthQueryApplicationCommitDenial::invariant_execution_denied(
            WorthQueryApplicationCommitDenialStage::InvariantExecution,
            failure.clone(),
        );
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProviderRejected
        );
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::InvariantExecution
        );
        let retained = denial.invariant_execution_failure().unwrap();
        assert_eq!(retained, &failure);
        assert_eq!(retained.kind(), expected);
        assert_eq!(retained.posture(), Posture::Exhausted);
        assert_eq!(denial.detail(), Some(retained.detail()));
        assert_eq!(
            runtime
                .branch_reference_state(identity.branch_id())
                .unwrap(),
            before
        );
    }
}

#[test]
fn invariant_materialization_refusal_is_not_exhaustion() {
    for (owner_denial, expected_detail) in [
        (
            Staging::MaterializationAuthorityRequired,
            "Relational invariant transaction requires materialization authority",
        ),
        (
            Staging::MaterializationModeMismatch,
            "Relational invariant transaction materialization mode does not match its intents",
        ),
    ] {
        let failure = map_transaction_staging_failure(owner_denial);
        let denial = WorthQueryApplicationCommitDenial::invariant_execution_denied(
            WorthQueryApplicationCommitDenialStage::InvariantExecution,
            failure,
        );
        let retained = denial.invariant_execution_failure().unwrap();
        assert_eq!(retained.kind(), Kind::ProviderRejected);
        assert_eq!(retained.posture(), Posture::Denied);
        assert_eq!(retained.detail(), expected_detail);
    }
}

fn runtime_with_bounds(maximum_bytes: u64, maximum_loci: usize) -> RelationalRuntime {
    let mut policy = worth_relational::facade::runtime::RelationalRuntimeConfig::resolved(
        RelationalRuntimeProfile::AiWorkflow,
        Default::default(),
    )
    .publication
    .policy;
    policy.max_transaction_overlay_bytes = maximum_bytes;
    policy.max_transaction_footprint_loci = maximum_loci;
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .publication(policy)
        .build()
}
