//! Branch merge over named outputs (persistent naming Phase 7, Arch 5, 10).
//!
//! A Native branch merge adopts source records without rerunning the
//! producers that named them. Until producer-rerunning merge is active it
//! must refuse typed, before any effect, naming the occurrence it cannot
//! carry: a source-named record (a distinct birth, or one colliding with the
//! target's), or group lineage the source published after the merge base.
//! It never silently adopts a record unnamed or rebinds a name across
//! branches, and records the source never named still merge.
use super::*;
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::merge::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent,
};

fn fork(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> AdmittedRelationalBranchBasis {
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    let fork = runtime
        .fork_branch(execution, BranchId("feature".into()), source)
        .unwrap();
    runtime.observe_branch(fork.target_identity()).unwrap().1
}

fn feature_head(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    let identity = runtime
        .branch_identity(&BranchId("feature".into()))
        .unwrap();
    runtime.observe_branch(&identity).unwrap().1
}

fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Result<worth_relational::facade::merge::PreparedMergeExecution, MergeExecutionPreparationError>
{
    let bound = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".into()),
            BranchId("feature".into()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    runtime.prepare_merge_execution(execution, bound)
}

/// Preparing the merge refuses typed; the target head does not move.
fn refused(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> MergeExecutionPreparationError {
    let before = head(runtime).descriptor().clone();
    let refusal = prepare(execution, runtime).expect_err("a named-output merge refuses");
    assert_eq!(head(runtime).descriptor(), &before, "zero effects");
    refusal
}

fn live_name(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    entity: EntityId,
) -> PersistentNameKey {
    match name_of(execution, runtime, basis, entity) {
        PersistentNameOfRecord::Live { name, .. } => name,
        other => panic!("{entity:?} must be live and named: {other:?}"),
    }
}

fn expect_named_record(
    refusal: MergeExecutionPreparationError,
    record: EntityId,
    name: PersistentNameKey,
) {
    let MergeExecutionPreparationError::NamedRecord(source_branch, refused, refused_name) = refusal
    else {
        panic!("refusal must name the source record: {refusal:?}");
    };
    assert_eq!(source_branch, BranchId("feature".into()));
    assert_eq!(refused, RecordRef::Entity(record));
    assert_eq!(PersistentNameKey::Entity(refused_name), name);
}

#[test]
fn a_source_named_birth_refuses_naming_it_and_never_goes_live_unnamed() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    create_entity(execution, &runtime, "model");
    let feature = fork(execution, &runtime);
    let on_main = owned_entity(execution, &runtime, &head(&runtime), "main-born", true);
    let on_feature = owned_entity(execution, &runtime, &feature, "feature-born", true);
    let main_name = live_name(execution, &runtime, &head(&runtime), on_main);
    let feature_name = live_name(execution, &runtime, &feature_head(&runtime), on_feature);
    expect_named_record(refused(execution, &runtime), on_feature, feature_name);
    let after = head(&runtime);
    assert!(resolve(execution, &runtime, &after, feature_name).contains("UnknownName"));
    assert_eq!(live_name(execution, &runtime, &after, on_main), main_name);
    assert_eq!(
        live_name(execution, &runtime, &feature_head(&runtime), on_feature),
        feature_name,
        "the source branch keeps its own name"
    );
}

#[test]
fn a_colliding_birth_refuses_and_each_branch_keeps_its_own_holder() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    create_entity(execution, &runtime, "model");
    let feature = fork(execution, &runtime);
    // The same authored occurrence on both branches mints one identity.
    let on_main = owned_entity(execution, &runtime, &head(&runtime), "same", true);
    let on_feature = owned_entity(execution, &runtime, &feature, "same", true);
    let name = live_name(execution, &runtime, &head(&runtime), on_main);
    assert_eq!(
        live_name(execution, &runtime, &feature_head(&runtime), on_feature),
        name
    );
    expect_named_record(refused(execution, &runtime), on_feature, name);
    let main_answer = resolve(execution, &runtime, &head(&runtime), name);
    assert!(
        main_answer.contains("Exact") && main_answer.contains(&format!("{on_main:?}")),
        "never rebound to the other branch's holder: {main_answer}"
    );
}

#[test]
fn a_split_on_the_source_and_an_edit_on_the_target_refuses_typed() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    create_entity(execution, &runtime, "model");
    let parent = owned_entity(execution, &runtime, &head(&runtime), "parent", true);
    let name = live_name(execution, &runtime, &head(&runtime), parent);
    let feature = fork(execution, &runtime);
    // The source ends the parent and births its piece; the target edits on.
    let mut transaction = runtime
        .begin_branch_transaction(&feature, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("feature-split").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: parent }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let piece = owned_entity(execution, &runtime, &feature_head(&runtime), "piece", true);
    owned_entity(execution, &runtime, &head(&runtime), "main-edit", true);
    let refusal = refused(execution, &runtime);
    assert!(
        matches!(
            refusal,
            MergeExecutionPreparationError::NotExecutionReady(_)
                | MergeExecutionPreparationError::NamedRecord(..)
                | MergeExecutionPreparationError::NamedLineage(..)
        ),
        "{refusal:?}"
    );
    let after = head(&runtime);
    assert_eq!(live_name(execution, &runtime, &after, parent), name);
    let piece_name = live_name(execution, &runtime, &feature_head(&runtime), piece);
    assert!(resolve(execution, &runtime, &after, piece_name).contains("UnknownName"));
}

#[test]
fn source_group_lineage_after_the_base_refuses_naming_its_commit() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    create_entity(execution, &runtime, "model");
    let member = owned_entity(execution, &runtime, &head(&runtime), "grouped", true);
    let name = live_name(execution, &runtime, &head(&runtime), member);
    let feature = fork(execution, &runtime);
    let PersistentNameKey::Entity(identity) = name else {
        panic!("a member is an entity name");
    };
    let mut plan = CommitNamingPlan::default();
    let scheme = AggregateGroupScheme::new(
        AggregateGroupSchemeId::new("loop").unwrap(),
        AggregateContinuityRule::Anchor,
    );
    plan.declare_group_partition(
        scheme,
        Vec::new(),
        vec![vec![(identity, AggregateMemberRole::Anchor)]],
    );
    let mut transaction = runtime
        .begin_branch_transaction(&feature, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("feature-loop").push(part("feature-part")),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.attach_naming_plan(plan);
    let committed = transaction.commit(execution, &runtime).unwrap();
    let grouped_at = committed.commit.commit_id;
    release_test_commit_snapshot(&runtime, &committed);
    let refusal = refused(execution, &runtime);
    assert_eq!(
        refusal,
        MergeExecutionPreparationError::NamedGroupLineage(
            BranchId("feature".into()),
            grouped_at,
            "loop".into(),
        )
    );
}

fn part(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId(3),
        kind_id: PART,
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::default(),
    }))
}

#[test]
fn records_the_source_never_named_still_merge() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    create_entity(execution, &runtime, "model");
    let named = owned_entity(execution, &runtime, &head(&runtime), "named", true);
    let name = live_name(execution, &runtime, &head(&runtime), named);
    let feature = fork(execution, &runtime);
    owned_entity(execution, &runtime, &feature, "plain", false);
    let before = head(&runtime).descriptor().clone();
    let prepared = prepare(execution, &runtime).expect("an unnamed adoption prepares");
    runtime
        .execute_prepared_merge(execution, prepared)
        .expect("an unnamed adoption merges");
    let after = head(&runtime);
    assert_ne!(after.descriptor(), &before, "the merge committed");
    assert_eq!(live_name(execution, &runtime, &after, named), name);
    assert!(resolve(execution, &runtime, &after, name).contains("Exact"));
}

fn identity(key: PersistentNameKey) -> worth_relational::facade::lineage::PersistentNameIdentity {
    match key {
        PersistentNameKey::Entity(identity) => identity,
        other => panic!("a member is an entity name: {other:?}"),
    }
}

#[test]
fn a_source_keep_in_place_merge_over_a_target_retirement_refuses_typed() {
    use worth_relational::facade::lineage::LineageMergeKeeper;
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    create_entity(execution, &runtime, "model");
    let keeper = owned_entity(execution, &runtime, &head(&runtime), "keeper", true);
    let absorbed = owned_entity(execution, &runtime, &head(&runtime), "absorbed", true);
    let kept = live_name(execution, &runtime, &head(&runtime), keeper);
    let gone = live_name(execution, &runtime, &head(&runtime), absorbed);
    let feature = fork(execution, &runtime);
    delete(execution, &runtime, &[absorbed]);
    let mut transaction = runtime
        .begin_branch_transaction(&feature, RelationalTransactionIntent::ordinary())
        .unwrap();
    let removal = DeleteEntityIntent {
        entity_id: absorbed,
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new("feature-merge").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(removal),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let parent = |entity, name| (LineageRecordTarget::Entity(entity), identity(name));
    transaction.attach_naming_plan(CommitNamingPlan {
        postures: vec![LineagePostureClaim::Merge {
            parents: vec![parent(keeper, kept), parent(absorbed, gone)],
            target: LineageRecordTarget::Entity(keeper),
            keeper: LineageMergeKeeper::KeepParent(identity(kept)),
        }],
        ..CommitNamingPlan::default()
    });
    let committed = transaction.commit(execution, &runtime).unwrap();
    let merged_at = committed.commit.commit_id;
    release_test_commit_snapshot(&runtime, &committed);
    let on_feature = resolve(execution, &runtime, &feature_head(&runtime), gone);
    assert!(on_feature.contains(&format!("{keeper:?}")), "{on_feature}");
    let refusal = refused(execution, &runtime);
    assert_eq!(
        refusal,
        MergeExecutionPreparationError::NamedLineage(BranchId("feature".into()), merged_at),
        "the source's absorbed -> keeper edge is never dropped silently"
    );
}
