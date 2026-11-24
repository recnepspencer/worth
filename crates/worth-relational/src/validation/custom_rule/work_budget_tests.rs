use std::num::NonZeroU64;
use std::sync::Arc;

use super::{structural_views::StructuralRelationView, CustomInvariantWorkMeter};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::storage::overlay::PartitionState;
use crate::validation::data::descriptor::CustomInvariantAccessContract;
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::BTreeMap;

#[test]
fn adjacency_budget_charges_candidates_even_when_access_filters_every_relation() {
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, source) = relation_budget_state();
    let state = InvariantStateView::new(&partitions, VersionId(1));
    let work = CustomInvariantWorkMeter::new(Some(NonZeroU64::new(4).unwrap()));
    let relations = StructuralRelationView::new(
        state,
        *execution,
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
    let host_request = crate::execution::test_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (partitions, source) = relation_budget_state();
    let state = InvariantStateView::new(&partitions, VersionId(1));
    let work = CustomInvariantWorkMeter::new(Some(NonZeroU64::new(3).unwrap()));
    let relations = StructuralRelationView::new(
        state,
        *execution,
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
    use crate::config::data::runtime_config::RelationalRuntimeConfig;
    use crate::runtime::state::subsystems::schema_contract_runtime::SchemaContractRuntimeSubsystem;
    use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
    use crate::validation::engine::{
        InvariantObservation, InvariantRuntimeInputs, InvariantRuntimeView,
    };
    use worth_execution::{CancellationSource, ExecutionRequest, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    let (partitions, source) = relation_budget_state();
    let observation = InvariantObservation::committed_branch(&partitions);
    let current = VersionId(1);
    let config = Arc::new(RelationalRuntimeConfig::default());
    let contracts = SchemaContractRuntimeSubsystem::for_schema(
        &config.schema.registry,
        &[],
        Default::default(),
        Default::default(),
        1,
        true,
    );
    let instrumentation = RuntimeInstrumentation::new();
    let view = InvariantRuntimeView::from_inputs(InvariantRuntimeInputs {
        selected_registry: crate::schema::SchemaRegistryCustody::Runtime(config.clone()),
        prior_registry: None,
        commit_work_budget: None,
        config,
        schema_contract_runtime: Arc::new(contracts),
        instrumentation: &instrumentation,
        current_version_id: current,
        entity_count: 2,
        relation_count: 1,
        version_depth: 1,
        snapshot_pressure: false,
    });
    for leased in [false, true] {
        for expected in [Ok(()), Err(Stop::Cancelled), Err(Stop::DeadlineElapsed)] {
            let cancellation = CancellationSource::new();
            let deadline = (expected == Err(Stop::DeadlineElapsed)).then(|| {
                let clock = worth_execution::MonotonicClock::system();
                clock.deadline_at(clock.now())
            });
            let serial = crate::execution::test_policy::serial_request()
                .with_cancellation(cancellation.token())
                .with_deadline(deadline.clone());
            let lease = crate::execution::test_authority::test_execution_authority()
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
            let scope =
                PreparedCustomInvariantScope::capture(&observation, current, None, &access, &work);
            let planner = CustomInvariantScopePlanner::new_at_current_version(
                &view,
                execution,
                &observation,
                current,
                current,
                &scope,
                work.clone(),
                access.clone(),
            );
            let context = CustomInvariantExecutionContext::new(
                &view,
                execution,
                &observation,
                current,
                current,
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

fn relation_budget_state() -> (BTreeMap<PartitionId, PartitionState>, EntityId) {
    let id = PartitionId::main();
    let graph =
        crate::storage::overlay::small_relation_partition::small_relation_partition(id, true);
    (BTreeMap::from([(id, graph.partition)]), graph.source)
}
