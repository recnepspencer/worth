//! An independent publication may not sever the source of an opted-in child.
use super::*;
fn meaning(family: &str, source_scope: EntityId) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: family.into(),
        source_scope,
        source_partition: [0; 32],
        source_epoch: [1; 32],
    }
}
fn role(
    target: EntityReference,
    posture: RequiredOutputRolePosture,
    index: usize,
) -> RequiredOutputRoleTarget {
    RequiredOutputRoleTarget {
        role: format!("output-{index}"),
        entity_name: "node".into(),
        kind: KindId(1),
        posture,
        target,
    }
}
#[test]
fn independent_parent_cannot_retire_or_replace_the_source_of_a_live_owned_child() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = registration::runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed =
        installed.with_source_scope_owned_producers([("owned-child".into(), KindId(1))]);
    let target = targets(&["ordinary-source"]);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(creates("author", &target), execution.allocation_policy())
        .unwrap();
    let born = transaction.commit(execution, &runtime).unwrap();
    let source = born.created_entity(&target[0]).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let parent_targets = targets(&["independent-child-source"]);
    let parent_meaning = meaning("independent-parent", source);
    let parent = publish(
        &runtime,
        execution,
        &installed,
        creates("independent-parent", &parent_targets),
        RequiredOutputPublication::Birth {
            admission: owner.admit_required_output("independent-parent", [1; 32]),
            absence: None,
            meaning: parent_meaning.clone(),
            roles: vec![role(
                EntityReference::Created(parent_targets[0].clone()),
                RequiredOutputRolePosture::Create,
                0,
            )],
        },
    );
    let child_source = parent.created_entity(&parent_targets[0]).unwrap();
    release_test_commit_snapshot(&runtime, &parent);
    let retirement = vec![role(
        EntityReference::Existing(child_source),
        RequiredOutputRolePosture::Retire,
        0,
    )];
    let basis = || {
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
    };
    let delete_source = || {
        WorkerIntentBatch::new("retire-independent-parent").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: child_source,
            }),
        ))
    };
    // Before an owned child exists the complete legacy retirement still admits,
    // even though another opted-in producer is installed. Drop without publishing.
    let prior = runtime
        .readmit_authored_output_retirement(execution, &installed, &basis(), &retirement)
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_authored_output_retirement(prior.authority())
        .unwrap();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(delete_source(), execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, &runtime)
        .unwrap()
        .bind_required_output_publication(
            &installed,
            RequiredOutputPublication::AuthoredRetirement {
                admission,
                predecessor: prior,
                roles: retirement.clone(),
            },
        )
        .unwrap();
    drop(
        runtime
            .prepare_validated_proposal(execution, proposal)
            .expect("unchanged independent path"),
    );
    let child_targets = targets(&["owned-realization", "owned-cell"]);
    let child = publish(
        &runtime,
        execution,
        &installed,
        creates("owned-child", &child_targets),
        RequiredOutputPublication::Birth {
            admission: owner.admit_required_output("owned-child", [1; 32]),
            absence: None,
            meaning: meaning("owned-child", child_source),
            roles: child_targets
                .iter()
                .enumerate()
                .map(|(index, target)| {
                    role(
                        EntityReference::Created(target.clone()),
                        RequiredOutputRolePosture::Create,
                        index,
                    )
                })
                .collect(),
        },
    );
    let child_ids = child_targets
        .iter()
        .map(|target| child.created_entity(target).unwrap())
        .collect::<Vec<_>>();
    let child_publication = child.commit.commit_id;
    release_test_commit_snapshot(&runtime, &child);
    let before = basis();
    let current_parent = || {
        runtime
            .readmit_required_output_publication(
                execution,
                &installed,
                &before,
                "independent-parent",
                &[1; 32],
                &parent_meaning,
                &[role(
                    EntityReference::Existing(child_source),
                    RequiredOutputRolePosture::Create,
                    0,
                )],
            )
            .unwrap()
    };
    owned_routes::refused(
        runtime
            .readmit_authored_output_retirement(execution, &installed, &before, &retirement)
            .unwrap_err(),
        child_source,
        child_publication,
        false,
    );
    owned_routes::refused(
        runtime
            .refuse_owned_output_retirement(execution, &installed, &before, &retirement)
            .unwrap_err(),
        child_source,
        child_publication,
        false,
    );
    let prior = current_parent();
    let admission = owner
        .admit_authored_output_retirement(prior.authority())
        .unwrap();
    let error = owned_routes::prepare_refusal(
        execution,
        &runtime,
        &installed,
        delete_source(),
        RequiredOutputPublication::AuthoredRetirement {
            admission,
            predecessor: prior,
            roles: retirement.clone(),
        },
    );
    owned_routes::refused(error, child_source, child_publication, false);
    let prior = current_parent();
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("independent-parent", [2; 32]),
            prior.authority(),
        )
        .unwrap();
    let replacement = targets(&["replacement-independent-source"])[0].clone();
    let batch = creates(
        "replace-independent-source",
        std::slice::from_ref(&replacement),
    )
    .push(MutationIntent::Entity(EntityMutationIntent::Delete(
        DeleteEntityIntent {
            entity_id: child_source,
        },
    )));
    let mut roles = retirement;
    roles.push(role(
        EntityReference::Created(replacement),
        RequiredOutputRolePosture::Create,
        1,
    ));
    let error = owned_routes::prepare_refusal(
        execution,
        &runtime,
        &installed,
        batch,
        RequiredOutputPublication::Preserve {
            admission,
            predecessor: prior,
            meaning: parent_meaning,
            roles,
        },
    );
    owned_routes::refused(error, child_source, child_publication, false);
    assert_eq!(basis().descriptor(), before.descriptor());
    // Hand-worked end state: S, independent Sc, and owned {R,C}: four live rows.
    let view = runtime
        .read_truth()
        .project_observation(&before.observation())
        .unwrap();
    for id in [source, child_source, child_ids[0], child_ids[1]] {
        assert!(view
            .read_exact_entity_state(execution, id, |_, _| ())
            .unwrap()
            .is_some());
    }
}
