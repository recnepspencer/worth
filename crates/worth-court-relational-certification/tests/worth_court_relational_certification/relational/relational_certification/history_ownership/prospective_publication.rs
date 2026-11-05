//! Exclusivity judges custody that the publication writer first installs in this candidate.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::mvcc::*;
#[test]
fn a_publications_first_candidate_cannot_give_its_new_member_two_document_owners() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let foreign = create_entity(execution, &world.runtime, "other-document");
    let source = member(execution, &world.runtime, "second-source");
    create_relation(
        execution,
        &world.runtime,
        world.model,
        source,
        "owns-second-source",
    );
    let mut meaning = world.meaning.clone();
    meaning.source_scope = source;
    meaning.source_epoch = [2; 32];
    let output = CreatedEntityRef {
        partition_id: PartitionId(4),
        kind_id: KindId(3),
        client_key: ClientKey::raw("new-output"),
    };
    let mut tx = begin(&world.runtime);
    tx.push_batch(
        WorkerIntentBatch::new("producer-and-foreign-owner")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: output.partition_id,
                kind_id: output.kind_id,
                client_key: output.client_key.clone(),
                fields: AspectFieldPatch::default(),
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("foreign-owner-of-new-output"),
                    source: EntityReference::Existing(foreign),
                    target: EntityReference::Created(output.clone()),
                    fields: AspectFieldPatch::default(),
                },
            ))),
        execution.allocation_policy(),
    )
    .unwrap();
    let (_, before) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let absence = world
        .runtime
        .readmit_required_output_route_absence(execution, &world.installed, &before, &meaning)
        .unwrap();
    let proposal = tx
        .validate(execution, &world.runtime)
        .unwrap()
        .bind_required_output_publication(
            &world.installed,
            RequiredOutputPublication::Birth {
                admission: world.owner.admit_required_output("generator", [2; 32]),
                absence,
                meaning,
                roles: vec![RequiredOutputRoleTarget {
                    role: "result".into(),
                    entity_name: "Member".into(),
                    kind: KindId(3),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Created(output),
                }],
            },
        )
        .unwrap();
    let error = world
        .runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("{error:?}")
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::CustomInvariantViolation {
                refusal:
                    worth_relational::facade::runtime::CustomInvariantRefusal::DeclaredValue {
                        value_identity,
                        bytes,
                    },
                ..
            },
        ..
    } = error.class
    else {
        panic!("{error:?}")
    };
    assert_eq!(
        value_identity,
        worth_relational::facade::branch::RelationalModelOwnershipConflict::VALUE_IDENTITY
    );
    let conflict =
        worth_relational::facade::branch::RelationalModelOwnershipConflict::decode(&bytes).unwrap();
    assert_eq!(conflict.subject.partition_id, PartitionId(4));
    assert_ne!(conflict.subject, world.output);
    assert!(begin(&world.runtime)
        .read_entity(execution, conflict.subject)
        .unwrap()
        .base()
        .is_none());
    assert_eq!(
        world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
}
