use crate::contracts::identity::data::primitives::{KindId, PartitionId, VersionId};
use crate::contracts::symbols::data::client_key::ClientKey;
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, DeleteEntityIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
    ReplaceEntityIntent, UpdateRelationEndpointsIntent,
};
use crate::contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::contracts::transactions::data::primitives::{
    EntityReference, EntitySpec, RelationSpec, TransactionId,
};
use crate::validation::engine::{
    test_view::empty_invariant_view, InvariantExecutionRequest, InvariantObservation,
    InvariantPreparationControl, InvariantRequestProfile,
};
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::small_relation_partition::typed_relation_partition_for_test;

use super::CandidateTouchedKinds;

#[test]
fn replacement_defers_old_kind_and_adjacency_to_the_ordinary_planner() {
    let (partitions, entities, _) = partitions([KindId(2)], &[]);
    let entity = entities[0];
    let plan = plan(
        1,
        MutationIntent::Entity(EntityMutationIntent::Replace(ReplaceEntityIntent {
            entity_id: entity,
            replacement: EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: ClientKey::raw("replacement"),
                fields: Default::default(),
            },
        })),
    );
    assert!(touched(&partitions, &plan).is_none());
}

#[test]
fn endpoint_move_defers_old_source_kind_to_the_ordinary_planner() {
    let (partitions, entities, relations) =
        partitions([KindId(2), KindId(1), KindId(3)], &[(0, 1, KindId(4))]);
    let target = entities[1];
    let new_source = entities[2];
    let relation = relations[0];
    let plan = plan(
        2,
        MutationIntent::Relation(RelationMutationIntent::UpdateEndpoints(
            UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(4),
                source: EntityReference::Existing(new_source),
                target: EntityReference::Existing(target),
            },
        )),
    );
    assert!(touched(&partitions, &plan).is_none());
}

#[test]
fn entity_delete_defers_cascade_applicability_to_the_ordinary_planner() {
    let (partitions, entities, _) = partitions([KindId(1), KindId(1)], &[(0, 1, KindId(4))]);
    let source = entities[0];
    let plan = plan(
        3,
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id: source,
        })),
    );
    assert!(touched(&partitions, &plan).is_none());
}

#[test]
fn endpoint_touch_policy_preserves_default_and_allows_explicit_opt_out() {
    let (partitions, entities, _) = partitions([KindId(2), KindId(1)], &[]);
    let (source, target) = (entities[0], entities[1]);
    let plan = plan(
        4,
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw("unrelated-edge"),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
            fields: Default::default(),
        })),
    );
    let kinds = touched(&partitions, &plan).expect("create has a complete footprint");
    let mut access =
        crate::contracts::validation::data::descriptor::CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(2)],
            read_relation_kinds: Vec::new(),
            affected_entity_kinds: vec![KindId(2)],
            affected_relation_kinds: Vec::new(),
            include_relation_endpoint_entity_touches: true,
        };
    assert!(kinds.may_affect(&access));
    access.include_relation_endpoint_entity_touches = false;
    assert!(!kinds.may_affect(&access));
    access.read_relation_kinds.push(KindId(4));
    assert!(!kinds.may_affect(&access));
    access.read_relation_kinds.clear();
    access.affected_relation_kinds.push(KindId(4));
    assert!(kinds.may_affect(&access));
}

#[test]
fn direct_entity_creation_still_admits_entity_rule() {
    let partitions = std::collections::BTreeMap::new();
    let plan = plan(
        5,
        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("direct"),
            fields: Default::default(),
        })),
    );
    let kinds = touched(&partitions, &plan).expect("create has a complete footprint");
    assert!(kinds.may_affect(
        &crate::contracts::validation::data::descriptor::CustomInvariantAccessContract {
            read_entity_kinds: vec![KindId(2)],
            read_relation_kinds: Vec::new(),
            affected_entity_kinds: vec![KindId(2)],
            affected_relation_kinds: Vec::new(),
            include_relation_endpoint_entity_touches: false,
        }
    ));
}

fn touched(
    state: &dyn CurrentPartitionAccess,
    plan: &MergedCommitPlan,
) -> Option<CandidateTouchedKinds> {
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let instrumentation = worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let view = empty_invariant_view(&instrumentation);
    let version = VersionId(1);
    let request = InvariantExecutionRequest::from_profile_with_contract_at_current_version(
        &execution,
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(state),
        version,
        version,
        Some(plan),
        None,
        &InvariantPreparationControl::uninterrupted_proposal_validation(),
    )
    .unwrap();
    CandidateTouchedKinds::from_request(&request, None)
}

fn plan(transaction: u64, intent: MutationIntent) -> MergedCommitPlan {
    MergedCommitPlan {
        transaction_id: TransactionId(transaction),
        merged_intents: vec![intent],
    }
}

fn partitions(
    kinds: impl IntoIterator<Item = KindId, IntoIter: ExactSizeIterator>,
    edges: &[(usize, usize, KindId)],
) -> (
    impl CurrentPartitionAccess,
    Vec<crate::contracts::identity::data::record_ids::EntityId>,
    Vec<crate::contracts::identity::data::record_ids::RelationId>,
) {
    let id = PartitionId::main();
    let (partition, entities, relations) =
        typed_relation_partition_for_test(id, kinds.into_iter(), edges);
    (
        std::collections::BTreeMap::from([(id, partition)]),
        entities,
        relations,
    )
}
