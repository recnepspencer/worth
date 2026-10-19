use std::num::NonZeroU64;
use std::sync::Arc;

use super::{structural_views::StructuralRelationView, CustomInvariantWorkMeter};
use crate::tests::support::{create_entity, create_relation, runtime_with_test_schema};
use crate::validation::data::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;

#[test]
fn adjacency_budget_charges_candidates_even_when_access_filters_every_relation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    create_relation(execution, &runtime, source, target, "edge");
    let observation = crate::validation::engine::InvariantObservation::committed(
        runtime.storage_access().current_edition(),
    );
    let state = InvariantStateView::new(
        observation.committed_partition_access(),
        runtime.current_version_id(),
    );
    let work = CustomInvariantWorkMeter::new(Some(NonZeroU64::new(4).unwrap()));
    let relations = StructuralRelationView::new(
        state,
        work.clone(),
        Arc::new(CustomInvariantAccessContract::default()),
    );
    assert!(relations
        .outgoing_relations_for_entity(source)
        .unwrap()
        .is_empty());
    assert_eq!(work.consumed().get(), 4);
    assert!(!work.exceeded());
    assert!(relations.outgoing_relations_for_entity(source).is_err());
    assert!(work.exceeded());
}

#[test]
fn rejected_adjacency_budget_never_consumes_the_denied_scan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    create_relation(execution, &runtime, source, target, "edge");
    let observation = crate::validation::engine::InvariantObservation::committed(
        runtime.storage_access().current_edition(),
    );
    let state = InvariantStateView::new(
        observation.committed_partition_access(),
        runtime.current_version_id(),
    );
    let work = CustomInvariantWorkMeter::new(Some(NonZeroU64::new(3).unwrap()));
    let relations = StructuralRelationView::new(
        state,
        work.clone(),
        Arc::new(CustomInvariantAccessContract::default()),
    );
    assert!(relations.outgoing_relations_for_entity(source).is_err());
    assert_eq!(work.consumed().get(), 1);
    assert!(work.exceeded());
    assert_eq!(
        relations.entity_kind(source),
        Err(super::structural_views::StructuralReadError::WorkBudgetExceeded)
    );
    assert_eq!(work.consumed().get(), 1);
}

#[test]
fn pure_invariant_observers_preserve_original_serial_and_leased_interruption() {
    use super::{
        CustomInvariantExecutionContext, CustomInvariantInterruption as Stop,
        CustomInvariantScopePlanner, PreparedCustomInvariantScope,
    };
    use worth_execution::{CancellationSource, ExecutionRequest, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let host = crate::test_execution_policy::serial_request();
    let runtime = runtime_with_test_schema();
    let source = create_entity(
        &ExecutionRequest::serial(&host),
        &runtime,
        "observed-source",
    );
    let observation = crate::validation::engine::InvariantObservation::committed(
        runtime.storage_access().current_edition(),
    );
    let view = crate::validation::engine::InvariantRuntimeView::from_runtime(&runtime);
    for leased in [false, true] {
        for expected in [Ok(()), Err(Stop::Cancelled), Err(Stop::DeadlineElapsed)] {
            let cancellation = CancellationSource::new();
            let deadline = (expected == Err(Stop::DeadlineElapsed)).then(std::time::Instant::now);
            let serial = crate::test_execution_policy::serial_request()
                .with_cancellation(cancellation.token())
                .with_deadline(deadline);
            let lease = crate::tests::support::test_execution_authority()
                .request_lease(LeaseRequest {
                    policy: ExecutionRequestPolicy::new(
                        ExecutionPosture::Serial,
                        DeterminismContract::CanonicalBitwise,
                        ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(100_000)),
                    ),
                    cancellation: cancellation.token(),
                    deadline,
                })
                .unwrap();
            let execution = if leased {
                ExecutionRequest::leased(&lease)
            } else {
                ExecutionRequest::serial(&serial)
            };
            let access = Arc::new(CustomInvariantAccessContract {
                read_entity_kinds: vec![crate::identity::data::KindId(1)],
                ..Default::default()
            });
            let work = CustomInvariantWorkMeter::new(None);
            let scope = PreparedCustomInvariantScope::capture(
                &observation,
                runtime.current_version_id(),
                None,
                &access,
                &work,
            );
            let planner = CustomInvariantScopePlanner::new_at_current_version(
                &view,
                execution,
                &observation,
                runtime.current_version_id(),
                runtime.current_version_id(),
                &scope,
                work.clone(),
                access.clone(),
            );
            let context = CustomInvariantExecutionContext::new(
                &view,
                execution,
                &observation,
                runtime.current_version_id(),
                runtime.current_version_id(),
                &scope,
                None,
                work.clone(),
                access,
            );
            assert_eq!(
                context.relations().entity_kind(source).unwrap(),
                crate::identity::data::KindId(1)
            );
            if expected == Err(Stop::Cancelled) {
                cancellation.cancel();
            }
            let charged = work.charged_work();
            assert_eq!(planner.check_live(), expected);
            assert_eq!(context.check_live(), expected);
            let cancelled = || context.check_live().is_err();
            assert_eq!(cancelled(), expected.is_err());
            assert_eq!(
                work.charged_work(),
                charged,
                "pure observers cannot charge rule work"
            );
            assert_eq!(lease.charged_memory_bytes(), 0);
        }
    }
}
