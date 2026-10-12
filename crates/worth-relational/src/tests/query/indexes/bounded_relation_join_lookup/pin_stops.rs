//! A pin refusal reaches the installed join door by value.
use super::*;
#[test]
fn relation_join_pin_keeps_the_cancelled_request_cause() {
    let host = crate::test_execution_policy::serial_request();
    let setup = worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let left = create_entity(&setup, &runtime, "left");
    let right = create_entity(&setup, &runtime, "right");
    let shared = create_entity(&setup, &runtime, "shared");
    bind_chain(&setup, &runtime, left, shared, right, "shared");
    let index = register_relation_join(&runtime, 81);
    build_current_generation(&setup, &runtime, index.index_id);
    let snapshot = runtime.visibility_authority().snapshot();
    for leased in [false, true] {
        let cancellation = worth_execution::CancellationSource::new();
        let stopped = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(64 << 20),
            cancellation.token(),
            None,
        );
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(worth_execution::LeaseRequest {
                policy: worth_foundational::ExecutionRequestPolicy::new(
                    worth_foundational::ExecutionPosture::Automatic,
                    worth_foundational::DeterminismContract::CanonicalBitwise,
                    worth_foundational::ExecutionBudget::new(
                        std::num::NonZeroUsize::new(1).unwrap(),
                        64 << 20,
                        100_000,
                    ),
                ),
                deadline: None,
                cancellation: cancellation.token(),
            })
            .unwrap();
        let request = if leased {
            worth_execution::ExecutionRequest::leased(&lease)
        } else {
            worth_execution::ExecutionRequest::serial(&stopped)
        };
        cancellation.cancel();
        let denial = runtime
            .index_access()
            .execute_bounded_relation_join_lookup(
                join_request(snapshot.clone(), index.index_id, left, right, 2),
                BoundedIndexParityMode::Production,
                &request,
            )
            .unwrap_err();
        assert_eq!(
            denial,
            crate::indexes::data::BoundedRelationJoinLookupDenial::new(
                BoundedRelationJoinLookupDenialKind::Execution(
                    crate::execution::RelationalExecutionDenialCause::Cancelled
                ),
                index.index_id,
            )
        );
    }
}
