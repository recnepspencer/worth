//! The receiving comparison consumes real retirement metadata at both roots.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, publish_relational_mutation, AccountIdentity,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
use worth_relational::facade::{
    history::{RelationalAuthoredWriteScope, RelationalAuthoredWriteSelector},
    identity::{EntityId, KindId, VersionId},
    transactions::{
        DeleteEntityIntent, EntityMutationIntent, MutationIntent, RecordRef, WorkerIntentBatch,
    },
};

fn payload(fact: Fact) -> FactPayload {
    std::sync::Arc::<[Fact]>::from(vec![fact]).into()
}

#[test]
fn paired_retirement_support_checks_original_and_target_exact_metadata() {
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
        let execution = phase.borrowed_request_for_test();
        let host = phase.test_host_request();
        let custody = host.phase_for_application(&world.application);
        let before = world.selected_product(&host);
        let entity = before.resolve_entity(
            &custody, AccountIdentity::reference(), "account-2".into(), &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap().entity_id();
        let other = before.resolve_entity(
            &custody, AccountIdentity::reference(), "account-1".into(), &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        ).unwrap().entity_id();
        let delete = |entity| WorkerIntentBatch::new("retirement-support").push(
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: entity }))
        );
        publish_relational_mutation(&execution, &world, delete(entity));
        let original = world.selected_product(&host);
        let graph = world.application.runtime.primary_graph().unwrap();
        let fact = graph.integration_handle().with_runtime(&execution, |runtime| {
            let view = runtime.read_truth().project_snapshot(original.application_basis().snapshot_handle()).unwrap();
            let retired = view.read_entity_retirement(&execution, entity).unwrap().unwrap();
            assert_eq!(retired.entity_id(), entity);
            Fact::RetiredOutputEntity {
                entity_id: entity, kind: retired.kind_id(),
                created_at: retired.created_at_version(), deleted_at: retired.deleted_at_version(),
                read_locator: Fact::Entity { entity_id: entity, kind: retired.kind_id() }.locator_identity(),
            }
        });
        // A real unrelated lifecycle publication moves the selected head but
        // does not mutate this full-GEN retirement tuple.
        publish_relational_mutation(&execution, &world, delete(other));
        let target = world.selected_product(&host);
        graph.integration_handle().with_runtime(&execution, |runtime| {
            let before_snapshot = before.application_basis().snapshot_handle();
            let original_snapshot = original.application_basis().snapshot_handle();
            let target_snapshot = target.application_basis().snapshot_handle();
            let original_basis = original.product().relational_basis();
            let target_basis = target.product().relational_basis();
            let target_commit = target_basis.observation().commit_id().unwrap();
            let limits = || RelationalAuthoredWritePageLimits {
                maximum_entries: NonZeroUsize::MIN,
                maximum_source_visits: NonZeroUsize::MIN,
                maximum_owned_bytes: NonZeroUsize::new(usize::MAX).unwrap(),
            };
            let original_view = runtime.read_truth().project_snapshot(original_snapshot).unwrap();
            let target_view = runtime.read_truth().project_snapshot(target_snapshot).unwrap();
            let facts = payload(fact.clone());
            let selection = overlap::select(execution, &facts, &original_view, &target_view).unwrap();
            let keys = selection.keys.iter().cloned().collect::<Vec<_>>();
            assert_eq!(keys, [RelationalAuthoredWriteSelector::Record {
                record: RecordRef::Entity(entity), scope: RelationalAuthoredWriteScope::Lifecycle,
            }]);
            let selected = runtime.select_exact_commit(&target_basis.observation(), target_commit)
                .into_outcome().unwrap();
            let mut cursor = runtime.authored_write_history_cursor(
                execution, original_basis, selected, &keys,
            ).unwrap();
            let page = cursor.next_page(runtime, execution, limits()).unwrap();
            assert!(page.is_complete());
            assert_eq!(page.entries().count(), 0, "the unrelated entity is not the retired output");
            let selected = runtime.select_exact_commit(&target_basis.observation(), target_commit)
                .into_outcome().unwrap();
            let mut crossing_retirement = runtime.authored_write_history_cursor(
                execution, before.product().relational_basis(), selected, &keys,
            ).unwrap();
            let page = crossing_retirement.next_page(runtime, execution, limits()).unwrap();
            assert!(page.is_complete());
            assert_eq!(page.entries().map(|entry| entry.commit_id()).collect::<Vec<_>>(),
                [original_basis.observation().commit_id().unwrap()],
                "the exact selector discovers the genuine deletion and no unrelated lifecycle");
            compare(runtime, execution, original_snapshot, target_snapshot, &facts).unwrap();
            assert!(matches!(
                compare(runtime, execution, original_snapshot, before_snapshot, &facts),
                Err(Denial::SourceDiverged)
            ), "a live original entity does not prove retirement");
            assert!(matches!(
                compare(runtime, execution, before_snapshot, target_snapshot, &facts),
                Err(Denial::SourceDiverged)
            ), "target retirement cannot establish a false original fact");
            let Fact::RetiredOutputEntity { kind, created_at, deleted_at, .. } = &fact else { unreachable!() };
            let mut wrong_generation = fact.clone();
            if let Fact::RetiredOutputEntity { entity_id, .. } = &mut wrong_generation {
                *entity_id = EntityId::new(entity.partition_id, entity.local_slot_value(), entity.generation_value() + 1);
            }
            let mut wrong_kind = fact.clone();
            if let Fact::RetiredOutputEntity { kind: actual, .. } = &mut wrong_kind {
                *actual = KindId::new(kind.as_u32() + 1);
            }
            let mut wrong_creation = fact.clone();
            if let Fact::RetiredOutputEntity { created_at: actual, .. } = &mut wrong_creation {
                *actual = VersionId(created_at.as_u64() + 1);
            }
            let mut wrong_deletion = fact.clone();
            if let Fact::RetiredOutputEntity { deleted_at: actual, .. } = &mut wrong_deletion {
                *actual = VersionId(deleted_at.as_u64() + 1);
            }
            for forged in [wrong_generation, wrong_kind, wrong_creation, wrong_deletion] {
                assert!(matches!(compare(runtime, execution, original_snapshot, target_snapshot, &payload(forged)), Err(Denial::SourceDiverged)));
            }
            let cancelled = worth_execution::CancellationSource::new();
            let request = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None), cancelled.token(), None,
            );
            cancelled.cancel();
            let stopped = compare(runtime, ExecutionRequest::serial(&request), original_snapshot, target_snapshot, &facts);
            assert!(matches!(stopped, Err(Denial::DependencyComparison(cause))
                if cause.kind() == crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Cancelled),
                "a stopped request remains a typed stop, not source divergence");
        });
    }).unwrap();
}
