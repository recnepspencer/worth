//! Production planning carries live custody rather than reserving a full ceiling.
use super::*;
use crate::facade::runtime::{
    InvariantCatalog, InvariantRegistration, InvariantRule, RelationalRuntimeApi,
};
use crate::facade::schema::RelationalSchemaRegistry;
use crate::validation::engine::{InvariantObservation, InvariantRequestProfile};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn checked_planning_retains_actual_owned_storage_and_releases_on_success_and_refusal() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::MaxMergedIntents(100),
            )],
        })
        .build();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let view = InvariantRuntimeView::from_runtime(&runtime);
    for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
        for bytes in [128, 64 * 1024] {
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        posture,
                        DeterminismContract::CanonicalBitwise,
                        ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), bytes, 100_000),
                    ),
                    cancellation: CancellationToken::new(),
                    deadline: None,
                })
                .unwrap();
            let result = plan_checked_invariant_preparation(&view, &request, &lease);
            if bytes == 128 {
                assert!(
                    matches!(result, Err(PacketExecutionStop::Execution { .. })),
                    "an unfunded plan must be refused through the execution owner"
                );
            } else {
                let (plan, custody, _) = result.expect("the actual native plan is funded");
                assert_eq!(plan.packets.len(), 1);
                let retained = lease.charged_memory_bytes();
                assert!(
                    retained > 0 && retained < bytes,
                    "planning must hold its actual allocation, leaving room for input caches"
                );
                drop(plan);
                assert_eq!(lease.charged_memory_bytes(), retained);
                drop(custody);
            }
            assert_eq!(
                lease.charged_memory_bytes(),
                0,
                "planning custody releases on success and refusal"
            );
        }
    }
}
