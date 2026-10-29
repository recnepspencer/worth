//! Ordinary authored retirement carries complete relation custody into warm/cold reissue.
use super::*;
use crate::identity::data::RelationId;
#[path = "authored/refusal.rs"]
mod refusal;
#[path = "authored/request.rs"]
mod request;

fn birth_with_edge(
    runtime: &crate::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
) -> (CommitResult, [EntityId; 2], RelationId) {
    let (batch, targets) = creates(["authored-left", "authored-right"]);
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("authored-owned-edge"),
        source: EntityReference::Created(targets[0].clone()),
        target: EntityReference::Created(targets[1].clone()),
    };
    let batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: edge.partition_id,
            kind_id: edge.kind_id,
            client_key: edge.client_key.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            fields: relation_label_field_patch("owned edge"),
        },
    )));
    let born = publish(
        runtime,
        execution,
        installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("authored-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| born.created_entity(&target).unwrap());
    let relation = born.created_relation(&edge).unwrap();
    let membership = born
        .publication()
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap();
    assert_eq!(membership.outputs, ids);
    assert_eq!(membership.relations.len(), 1);
    assert_eq!(
        (
            membership.relations[0].id,
            membership.relations[0].source,
            membership.relations[0].target
        ),
        (relation, ids[0], ids[1])
    );
    (born, ids, relation)
}

fn retire_roles(ids: [EntityId; 2]) -> Vec<RequiredOutputRoleTarget> {
    crate::mvcc::native_roles_fixture(
        &ids.map(|id| (id, KindId(1))),
        RequiredOutputRolePosture::Retire,
    )
}

fn authored(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    predecessor: RelationalRequiredOutputPublication,
    roles: Vec<RequiredOutputRoleTarget>,
) -> RequiredOutputPublication {
    let admission = owner
        .admit_authored_output_retirement(predecessor.authority())
        .unwrap();
    RequiredOutputPublication::AuthoredRetirement {
        admission,
        predecessor,
        roles,
    }
}

fn deletes(ids: [EntityId; 2], relation: RelationId) -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("ordinary-authored-complete-retirement").push(
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id: relation,
        })),
    );
    for id in ids {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: id },
        )));
    }
    batch
}

#[test]
fn ordinary_authored_complete_retirement_reissues_only_through_exact_warm_and_cold_successors() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    for cold in [false, true] {
        let mut runtime = runtime_with_test_schema();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(crate::schema::data::RelationalSchemaRegistry::new())
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let (born, ids, edge) = birth_with_edge(&runtime, execution, &installed, &owner);
        let prior = born.required_output_publication(&installed).unwrap();
        release_test_commit_snapshot(&runtime, &born);
        let basis = test_owner_main_basis(&runtime).unwrap();
        let previous = runtime
            .readmit_authored_output_retirement(execution, &installed, &basis, &retire_roles(ids))
            .unwrap()
            .unwrap();
        let retired = publish(
            &runtime,
            execution,
            &installed,
            deletes(ids, edge),
            authored(&owner, previous, retire_roles(ids)),
        )
        .unwrap();
        let origin = retired
            .publication()
            .envelope
            .required_output_origin
            .as_ref()
            .unwrap();
        assert!(
            matches!(origin.publication, crate::history::data::CanonicalOutputPublication::AuthoredRetirement { predecessor } if predecessor == born.commit.commit_id)
        );
        assert_eq!(origin.producer, "authored-creator");
        assert_eq!(origin.dependency_identity, [1; 32]);
        assert_eq!(origin.outputs, ids);
        assert!(origin
            .members
            .iter()
            .all(|member| member.posture == RequiredOutputRolePosture::Retire));
        assert!(origin.relations.is_empty());
        let canonical = std::sync::Arc::clone(&retired.publication().envelope);
        release_test_commit_snapshot(&runtime, &retired);
        let saved = runtime
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap();
        let mut reopened = runtime_with_test_schema();
        let receipt = reopened
            .prepare_initial_schema_installation()
            .unwrap()
            .install(crate::schema::data::RelationalSchemaRegistry::new())
            .unwrap();
        reopened
            .durability_recovery()
            .restore_native_checkpoint(execution, &saved)
            .unwrap();
        let receipt = reopened
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (cold_owner, cold_installed) = receipt.issue_application_publication_owner();
        let (runtime, owner, installed) = if cold {
            (&reopened, &cold_owner, &cold_installed)
        } else {
            (&runtime, &owner, &installed)
        };
        let restored = runtime
            .history
            .canonical_envelope(retired.commit.commit_id)
            .unwrap();
        assert_eq!(
            restored.required_output_origin, canonical.required_output_origin,
            "canonical authored succession survives real checkpoint reconstruction"
        );
        let basis = test_owner_main_basis(runtime).unwrap();
        let meaning = crate::mvcc::native_meaning_fixture("authored-creator");
        assert!(
            runtime
                .readmit_required_output_route_absence(execution, installed, &basis, &meaning)
                .unwrap()
                .is_none(),
            "authored retirement keeps Birth closed"
        );
        let previous = runtime
            .readmit_authored_output_retirement_successor(execution, installed, &basis, &meaning)
            .unwrap()
            .unwrap();
        let (batch, targets) = creates(["reissued-left", "reissued-right"]);
        let replacement = publish(
            runtime,
            execution,
            installed,
            batch,
            completion(owner, previous, new_roles(&targets), 2),
        )
        .unwrap();
        let new_ids = targets.map(|target| replacement.created_entity(&target).unwrap());
        assert!(new_ids
            .iter()
            .all(|id| !ids.contains(id) && id.generation_value() != 0));
        let current = replacement
            .publication()
            .envelope
            .required_output_origin
            .as_ref()
            .unwrap();
        assert_eq!(current.outputs, new_ids);
        assert_eq!(
            current.birth, born.commit.commit_id,
            "new creations succeed, never reopen the birth"
        );
        release_test_commit_snapshot(runtime, &replacement);
        if !cold {
            let batch = new_ids.into_iter().fold(
                WorkerIntentBatch::new("stale-retirement-predecessor"),
                |batch, id| {
                    batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                        DeleteEntityIntent { entity_id: id },
                    )))
                },
            );
            let denied = publish(
                runtime,
                execution,
                installed,
                batch,
                authored(owner, prior, retire_roles(new_ids)),
            )
            .unwrap_err();
            assert!(
                format!("{denied:?}").contains("not the publication selected"),
                "{denied:?}"
            );
        }
    }
}

#[test]
fn relation_refusal_returns_exact_predecessor_relation_origin_and_before_candidate_context() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let (born, ids, edge) = birth_with_edge(&runtime, execution, &installed, &owner);
    let previous = born.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let mut tx = test_owner_begin_transaction_for_main(&runtime);
    tx.push_batch(
        WorkerIntentBatch::new("ordinary-unlink-does-not-establish-retirement-succession").push(
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: edge,
            })),
        ),
        execution.allocation_policy(),
    )
    .unwrap();
    let unlinked = tx.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &unlinked);
    let before = test_owner_main_basis(&runtime).unwrap();
    let denied = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("unclassified-predecessor-relation-retirement"),
        preserve(&owner, previous, ids, 2),
    )
    .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = denied else {
        panic!("the actual relation rejection must retain publication cause")
    };
    assert!(
        error
            .detail
            .contains("publication relation generation absent or retired"),
        "{error:?}"
    );
    assert!(
        error.detail.contains("publication_stage=predecessor"),
        "{error:?}"
    );
    assert!(
        error.detail.contains(&format!("relation={edge:?}")),
        "{error:?}"
    );
    assert!(
        error.detail.contains(&format!("origin={:?}", born.commit)),
        "{error:?}"
    );
    assert!(
        error.detail.contains("before=") && error.detail.contains("candidate="),
        "{error:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        before.descriptor()
    );
}
