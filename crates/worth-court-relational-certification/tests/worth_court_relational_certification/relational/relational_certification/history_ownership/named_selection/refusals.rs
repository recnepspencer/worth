//! A selection derives its own continuity: it refuses a record this branch
//! never named of a schemed kind (selected from a sibling branch), and it
//! refuses any caller naming plan before admission.
use super::*;

fn fork(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> AdmittedRelationalBranchBasis {
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    let fork = runtime
        .fork_branch(execution, BranchId("sibling".into()), source)
        .unwrap();
    runtime.observe_branch(fork.target_identity()).unwrap().1
}

fn assert_refused_unchanged(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    model: EntityId,
    sibling: &AdmittedRelationalBranchBasis,
) {
    let before = head(runtime).descriptor().clone();
    let refused = select(execution, runtime, model, sibling)
        .expect_err("an unseen schemed record cannot go live unnamed");
    let rendered = format!("{refused:?}");
    assert!(rendered.contains("MissingBirth"), "{rendered}");
    assert_eq!(head(runtime).descriptor(), &before, "nothing committed");
}

#[test]
fn a_sibling_branchs_schemed_member_refuses_instead_of_going_live_unnamed() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    let model = create_entity(execution, &runtime, "model");
    let sibling = fork(execution, &runtime);
    let member = owned_entity(execution, &runtime, &sibling, "sibling-member", true);
    commit_on(
        execution,
        &runtime,
        &runtime.observe_branch(sibling.identity()).unwrap().1,
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw("sibling-owns"),
            source: EntityReference::Existing(model),
            target: EntityReference::Existing(member),
            fields: AspectFieldPatch::default(),
        })),
        None,
        "sibling-owns",
    );
    let sibling = runtime.observe_branch(sibling.identity()).unwrap().1;
    assert_refused_unchanged(execution, &runtime, model, &sibling);
    assert!(begin(&runtime)
        .read_entity(execution, member)
        .unwrap()
        .base()
        .is_none());
}

#[test]
fn a_sibling_branchs_schemed_reference_refuses_instead_of_going_live_unnamed() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    let model = create_entity(execution, &runtime, "model");
    let part = owned_entity(execution, &runtime, &head(&runtime), "part", false);
    let catalog = create_entity_in_partition(execution, &runtime, "catalog", PartitionId(5));
    create_relation(execution, &runtime, model, part, "owns-part");
    let sibling = fork(execution, &runtime);
    let reference = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(4),
        client_key: ClientKey::raw("sibling-reference"),
        source: EntityReference::Existing(part),
        target: EntityReference::Existing(catalog),
    };
    commit_on(
        execution,
        &runtime,
        &sibling,
        MutationIntent::Create(CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(4),
            client_key: ClientKey::raw("sibling-reference"),
            source: EntityReference::Existing(part),
            target: EntityReference::Existing(catalog),
            fields: AspectFieldPatch::default(),
        })),
        Some(PendingBirthTarget::CreatedRelation(reference)),
        "sibling-reference",
    );
    let sibling = runtime.observe_branch(sibling.identity()).unwrap().1;
    assert_refused_unchanged(execution, &runtime, model, &sibling);
}

#[test]
fn a_caller_naming_plan_refuses_admission_and_leaves_the_name_index_healthy() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    let model = create_entity(execution, &runtime, "model");
    let born_before = head(&runtime);
    let member = owned_entity(execution, &runtime, &head(&runtime), "member", true);
    create_relation(execution, &runtime, model, member, "owns-member");
    let PersistentNameOfRecord::Live { name, .. } =
        name_of(execution, &runtime, &head(&runtime), member)
    else {
        panic!("the member is born named");
    };
    let PersistentNameKey::Entity(identity) = name else {
        panic!("a member name is an entity name");
    };
    let graph = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &head(&runtime),
            &born_before,
            &named_ownership(),
            &named_ownership(),
            model,
        )
        .unwrap();
    let mut transaction = begin(&runtime);
    transaction.attach_naming_plan(CommitNamingPlan {
        postures: vec![LineagePostureClaim::Retire {
            target: LineageRecordTarget::Entity(member),
            name: identity,
        }],
        ..CommitNamingPlan::default()
    });
    let refused = runtime
        .preparation_port()
        .admit_owned_graph_selection(execution, transaction, graph)
        .expect_err("a selection admits no naming plan");
    assert!(
        matches!(
            refused,
            RelationalOwnedGraphSelectionPreparationError::TargetNotPristine
        ),
        "{refused:?}"
    );
    assert!(matches!(
        name_of(execution, &runtime, &head(&runtime), member),
        PersistentNameOfRecord::Live { .. }
    ));
    select(execution, &runtime, model, &born_before).expect("a plain selection still commits");
    assert!(matches!(
        name_of(execution, &runtime, &head(&runtime), member),
        PersistentNameOfRecord::Ended { .. }
    ));
}
