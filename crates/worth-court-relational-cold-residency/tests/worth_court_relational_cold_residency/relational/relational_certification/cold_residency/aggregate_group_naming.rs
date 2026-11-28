//! Automatic graph partition naming through publication and external recovery.
mod fixture;
mod read_denials;
mod rule_reads;
mod schema;

use self::fixture::{commit, connect, create, created, read, relation, scope, start};
use self::schema::{CASES, COUPLING, VERTEX};
use super::bounded_naming_fixture::{durable_runtime_with, observe_branch};
use super::current_read_basis::snapshot_for_owner_branch;
use super::naming_branch_root_and_dissolve::fork_retained_at;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::{
    AggregateGroupChange, AggregateGroupLineageEvent, AggregateGroupNaming, AggregateIdentity,
    CanonicalGroupMemberIdentity,
};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::lineage::{
    LineageRecordTarget, PersistentNameKey, PersistentNameOfRecord,
};
use worth_relational::facade::runtime::{
    PinnedAggregateLineageScope, PinnedAggregateReadScope, RelationalRuntimeApi,
};
use worth_relational::facade::transactions::{
    DeleteRelationIntent, EntityReference, MutationIntent, RelationMutationIntent,
    WorkerIntentBatch,
};
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

#[test]
fn projected_cases_share_a_group_without_colliding_across_birth_allocations() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let build = |named| {
        RelationalRuntimeApi::builder()
            .schema_registry(schema::registry(true, named))
            .build()
            .unwrap()
    };
    let first = build(true);
    let second = build(true);
    commit(
        request,
        &second,
        WorkerIntentBatch::new("shift-allocation").push(create(&created(VERTEX, "dummy"))),
        Vec::new(),
    );

    let left = start(request, &first, true, true);
    let right = start(request, &second, true, true);
    assert_ne!(left.commit.commit.commit_id, right.commit.commit.commit_id);
    assert_ne!(left.face, right.face);
    let incidence = first
        .with_persistent_names_at_snapshot(&left.commit.snapshot, |view| {
            view.name_of_record(LineageRecordTarget::Relation(left.incidence))
        })
        .unwrap();
    assert!(matches!(
        incidence,
        PersistentNameOfRecord::Live {
            name: PersistentNameKey::Relation(_),
            ..
        }
    ));
    let groups = left.commit.group_lineage().unwrap();
    let [(_, group)] = groups.as_slice() else {
        panic!("expected one automatic scheme")
    };
    let [assignment] = group.assignments.as_slice() else {
        panic!("expected one group")
    };
    let AggregateGroupChange::Born(members) = &assignment.change else {
        panic!("expected birth")
    };
    let cases: BTreeSet<_> = members
        .iter()
        .map(|(key, member)| {
            let CanonicalGroupMemberIdentity::ProjectedCase { case, .. } = key else {
                panic!("expected projected identity")
            };
            assert_eq!(member.carrier.carrier(), left.face);
            case.as_str()
        })
        .collect();
    assert_eq!(members.len(), 2);
    assert_eq!(cases, BTreeSet::from(CASES));
    for (runtime, born) in [(&first, left), (&second, right)] {
        assert_eq!(
            born.commit.group_lineage().unwrap()[0].1.assignments[0].identity,
            assignment.identity
        );
        for side in 0..2 {
            let group = read(request, runtime, &born.commit.snapshot, born.face, side);
            assert_eq!(group.identity(), assignment.identity);
            assert_eq!(group.len(), 2);
        }
    }

    let unnamed = build(false);
    let born = start(request, &unnamed, true, false);
    assert!(born.commit.group_lineage().unwrap().is_empty());
    let negative = read(request, &unnamed, &born.commit.snapshot, born.face, 0);
    let positive = read(request, &unnamed, &born.commit.snapshot, born.face, 1);
    assert_eq!(negative.identity(), positive.identity());
    assert_eq!(negative.len(), 2);
}

#[test]
fn physical_orbit_mints_from_its_entity_name_on_the_birth_commit() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let registry = schema::registry(false, true);
    let scheme = schema::descriptor(false).group_scheme(&registry).unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let born = start(request, &runtime, false, true);
    let name = runtime
        .with_persistent_names_at_snapshot(&born.commit.snapshot, |view| {
            view.name_of_record(LineageRecordTarget::Entity(born.face))
        })
        .unwrap();
    let PersistentNameOfRecord::Live {
        name: PersistentNameKey::Entity(name),
        ..
    } = name
    else {
        panic!("the commit must publish the carrier name")
    };
    let expected = AggregateIdentity::from_group_name(scheme.id(), name, scheme.rule());
    let pin = runtime
        .read_truth()
        .pin_read_capability(&born.commit.snapshot, &[], request)
        .unwrap();
    let group = pin
        .read_aggregate(request, PinnedAggregateReadScope::new(COUPLING, born.face))
        .unwrap();
    assert_eq!(group.identity(), expected);
    assert_eq!(group.members().collect::<Vec<_>>(), [born.face]);
    assert_eq!(
        born.commit.group_lineage().unwrap()[0].1.assignments[0].identity,
        expected
    );
}

fn disconnect(relation_id: RelationId) -> WorkerIntentBatch {
    WorkerIntentBatch::new("split").push(delete_relation(relation_id))
}

fn delete_relation(relation_id: RelationId) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id,
    }))
}

#[test]
fn canonical_group_keeper_and_revived_fragment_survive_cold_checkpoint_tail_replay() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let registry = schema::registry(true, true);
    let producer = durable_runtime_with(&medium, u64::MAX / 4, registry.clone());
    let main = producer.main_branch_identity().branch_id().clone();
    let born = start(request, &producer, true, true);
    let keeper = read(request, &producer, &born.commit.snapshot, born.face, 0).identity();

    let split = commit(request, &producer, disconnect(born.coupling), Vec::new());
    assert_eq!(
        read(request, &producer, &split.snapshot, born.face, 0).identity(),
        keeper
    );
    let fragment = read(request, &producer, &split.snapshot, born.face, 1).identity();
    assert_ne!(fragment, keeper);

    let coupling = relation(
        COUPLING,
        "reconnect",
        EntityReference::Existing(born.face),
        EntityReference::Existing(born.vertex),
    );
    let merged = commit(
        request,
        &producer,
        WorkerIntentBatch::new("merge").push(connect(&coupling)),
        Vec::new(),
    );
    assert!(merged.group_lineage().unwrap()[0].1.lineage.contains(
        &AggregateGroupLineageEvent::MergedInto {
            absorbed: fragment,
            keeper
        },
    ));
    let before_reform = BranchId("before-reform".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &before_reform,
    )
    .unwrap();
    let (checkpoint, _) = capture_external_checkpoint(request, &producer);
    let covered = checkpoint.covered_position().unwrap();
    let reformed = commit(
        request,
        &producer,
        disconnect(merged.created_relation(&coupling).unwrap()),
        Vec::new(),
    );
    assert!(reformed.group_lineage().unwrap()[0]
        .1
        .lineage
        .contains(&AggregateGroupLineageEvent::Revived { group: fragment },));
    assert_eq!(
        read(request, &producer, &reformed.snapshot, born.face, 1).identity(),
        fragment
    );
    assert_eq!(
        read(request, &producer, &merged.snapshot, born.face, 1).identity(),
        keeper
    );

    let source = immutable_source(&checkpoint, &medium);
    let checkpoint_source = RelationalDurableCheckpointSource::new(
        covered,
        checkpoint.covered_branch_transitions().unwrap(),
        checkpoint,
    );
    let tail = medium
        .committed_artifacts()
        .unwrap()
        .into_iter()
        .filter(|artifact| artifact.position() > covered)
        .collect();
    let input =
        RelationalDurableRecoverySource::new(Some(checkpoint_source), tail, Vec::new(), source);
    let mut cold = durable_runtime_with(&medium, u64::MAX / 4, registry);
    let plan = cold
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    cold.durability_recovery().recover(request, plan).unwrap();

    let snapshot = snapshot_for_owner_branch(&cold, &before_reform);
    let pin = cold
        .read_truth()
        .pin_read_capability(&snapshot, &[], request)
        .unwrap();
    let group = pin
        .read_projected_aggregate(request, scope(born.face, 1))
        .unwrap();
    assert_eq!(group.identity(), keeper);
    assert_eq!(group.len(), 2);
    let lineage = pin
        .read_aggregate_lineage(
            request,
            PinnedAggregateLineageScope::for_definition(
                schema::descriptor(true).definition,
                fragment,
            ),
        )
        .unwrap();
    assert_eq!(lineage.retired_at(), Some(merged.commit.commit_id));
    assert_eq!(lineage.latest_identity(), Some(keeper));
    assert_eq!(lineage.resolution_steps().count(), 1);
    let head = snapshot_for_owner_branch(&cold, &main);
    assert_eq!(
        read(request, &cold, &head, born.face, 1).identity(),
        fragment
    );
}
