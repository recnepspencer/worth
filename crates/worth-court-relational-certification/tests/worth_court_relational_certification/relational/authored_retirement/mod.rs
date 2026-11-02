//! Atomic source/realization retirement through the public Native boundary.
mod independent_source;
mod live_members;
mod owned_routes;
mod refusals;
mod restoration;
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::*,
    runtime::*,
    schema::RelationalSchemaRegistry,
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key,
    publications::{
        begin_main_transaction, release_test_commit_snapshot, string_aspect_field_patch,
    },
    AspectSchemaFixture,
};

fn fields(value: &str) -> AspectFieldPatch {
    string_aspect_field_patch([(aspect_key("name"), field_key("name"), value)])
}

fn targets(keys: &[&str]) -> Vec<CreatedEntityRef> {
    keys.iter()
        .map(|key| CreatedEntityRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: ClientKey::raw(*key),
        })
        .collect()
}

fn creates(producer: &str, targets: &[CreatedEntityRef]) -> WorkerIntentBatch {
    targets
        .iter()
        .fold(WorkerIntentBatch::new(producer), |batch, target| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: target.partition_id,
                kind_id: target.kind_id,
                client_key: target.client_key.clone(),
                fields: fields(target.client_key.as_raw_str().unwrap()),
            })))
        })
}

fn publish(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> CommitResult {
    let restoring = matches!(
        &publication,
        RequiredOutputPublication::AuthoredCascadeRestoration { .. }
    );
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(installed, publication)
        .unwrap();
    if restoring {
        let complete = proposal.mutation_touches(execution).unwrap();
        assert!(complete
            .touches()
            .iter()
            .any(|touch| matches!(touch, ValidatedMutationTouch::LinkRelation { .. })));
        let authored = proposal.application_mutation_touches(execution).unwrap();
        assert_eq!(
            authored.touches().len(),
            2,
            "only the source Create and its name field are application authored"
        );
        assert!(authored.touches().iter().any(|touch| matches!(touch, ValidatedMutationTouch::CreateEntity { kind } if *kind == KindId(1))));
        assert!(authored.touches().iter().any(|touch| matches!(touch, ValidatedMutationTouch::WriteEntityField { kind, .. } if *kind == KindId(1))));
        assert!(authored.struct_touches().is_empty());
    }
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("producer must publish")
    };
    runtime.settle_performed_publication(performed).unwrap()
}

fn require_atomic_retirement(depth: usize) {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    )
    .build_runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers(
        [
            "parent-realizer",
            "child-realizer",
            "leaf-realizer",
            "other-owner",
        ]
        .map(|name| (name.to_owned(), KindId(1))),
    );
    // The authored source is a genuine ordinary Create, not producer-owned
    // merely because a later producer reads it or creates an edge to it.
    let authored = targets(&["authored-source", "different-source", "ordinary-extra"]);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(creates("author", &authored), execution.allocation_policy())
        .unwrap();
    let born = transaction.commit(execution, &runtime).unwrap();
    let source = born.created_entity(&authored[0]).unwrap();
    let other_source = born.created_entity(&authored[1]).unwrap();
    let ordinary_extra = born.created_entity(&authored[2]).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let relation_ids = std::cell::RefCell::new(Vec::new());
    let meanings = std::cell::RefCell::new(Vec::new());
    let publication_ids = std::cell::RefCell::new(Vec::new());
    let birth = |producer: &str, keys: &[&str], source: EntityId| {
        let targets = targets(keys);
        let mut batch = creates(producer, &targets);
        for (index, target) in targets.iter().enumerate() {
            batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw(format!("{producer}-{index}")),
                    source: EntityReference::Created(target.clone()),
                    target: EntityReference::Existing(source),
                    fields: string_aspect_field_patch([(
                        aspect_key("label"),
                        field_key("label"),
                        "source association",
                    )]),
                },
            )));
        }
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let meaning = RequiredOutputPublicationMeaning {
            implementation: RequiredOutputProducingImplementation::from_checkpoint(
                [0x11; 16], [0x22; 32],
            )
            .unwrap(),
            family: producer.into(),
            source_scope: source,
            source_partition: [0; 32],
            source_epoch: [1; 32],
        };
        let absence = runtime
            .readmit_required_output_route_absence(execution, &installed, &basis, &meaning)
            .unwrap();
        assert!(absence.is_some(), "each genuine producer route starts once");
        meanings.borrow_mut().push(meaning.clone());
        let publication = RequiredOutputPublication::Birth {
            admission: owner.admit_required_output(producer, [1; 32]),
            absence,
            meaning,
            roles: targets
                .iter()
                .enumerate()
                .map(|(index, target)| RequiredOutputRoleTarget {
                    role: format!("output-{index}"),
                    entity_name: "node".into(),
                    kind: KindId(1),
                    posture: RequiredOutputRolePosture::Create,
                    target: EntityReference::Created(target.clone()),
                })
                .collect(),
        };
        let born = publish(&runtime, execution, &installed, batch, publication);
        publication_ids.borrow_mut().push(born.commit.commit_id);
        for (index, target) in targets.iter().enumerate() {
            relation_ids.borrow_mut().push(
                born.created_relation(&CreatedRelationRef {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw(format!("{producer}-{index}")),
                    source: EntityReference::Created(target.clone()),
                    target: EntityReference::Existing(source),
                })
                .unwrap(),
            );
        }
        let ids: Vec<_> = targets
            .iter()
            .map(|target| born.created_entity(target).unwrap())
            .collect();
        release_test_commit_snapshot(&runtime, &born);
        ids
    };
    let parent = birth(
        "parent-realizer",
        &["parent-realization", "child-source-or-cell"],
        source,
    );
    // Hand-worked ordinary roster: source; realization and its separate cell.
    let mut expected = vec![source, parent[0], parent[1]];
    if depth == 3 {
        let child = birth(
            "child-realizer",
            &["child-realization", "leaf-source"],
            parent[1],
        );
        let leaf = birth(
            "leaf-realizer",
            &["leaf-realization", "leaf-cell"],
            child[1],
        );
        // Hand-worked nested roster: source; {parent-realization, child-source};
        // {child-realization, leaf-source}; {leaf-realization, leaf-cell}.
        expected.extend([child[0], child[1], leaf[0], leaf[1]]);
    }
    assert_eq!(expected.len(), if depth == 3 { 7 } else { 3 });
    let selected_relations = relation_ids.borrow().clone();
    let selected_meanings = meanings.borrow().clone();
    let foreign = birth(
        "other-owner",
        &["foreign-realization", "foreign-cell"],
        other_source,
    );
    // Both producers read exactly the same source; neither opts into source ownership.
    let independent_a = birth("input-reader-a", &["shared-input-a"], source);
    let independent_b = birth("input-reader-b", &["shared-input-b"], source);

    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let roles: Vec<_> = expected
        .iter()
        .enumerate()
        .map(|(index, id)| RequiredOutputRoleTarget {
            role: format!("retired-{index}"),
            entity_name: "node".into(),
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Retire,
            target: EntityReference::Existing(*id),
        })
        .collect();
    refusals::check(
        execution,
        &runtime,
        &installed,
        source,
        &roles,
        ordinary_extra,
        foreign[0],
        independent_a[0],
    );
    owned_routes::check(
        execution,
        &runtime,
        &installed,
        &owner,
        &roles,
        &selected_meanings,
        &publication_ids.borrow()[..depth],
    );
    let result =
        runtime.readmit_authored_source_retirement(execution, &installed, &basis, source, &roles);
    let (_, after) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(after.descriptor(), basis.descriptor());
    assert!(result.as_ref().is_ok_and(Option::is_some),
        "complete atomic source/realization retirement must be admitted; roster={expected:?}; actual={result:?}");
    let predecessor = result.unwrap().unwrap();
    let admission = owner
        .admit_authored_output_retirements(predecessor.authorities())
        .unwrap();
    let mut batch = WorkerIntentBatch::new("atomic-authored-source-retirement");
    for id in selected_relations {
        batch = batch.push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id: id },
        )));
    }
    for id in &expected {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: *id },
        )));
    }
    let retired = publish(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::AuthoredCascadeRetirement {
            admission,
            predecessor,
        },
    );
    release_test_commit_snapshot(&runtime, &retired);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let view = runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    for id in [
        other_source,
        ordinary_extra,
        foreign[0],
        foreign[1],
        independent_a[0],
        independent_b[0],
    ] {
        assert!(
            view.read_exact_entity_state(execution, id, |_, _| ())
                .unwrap()
                .is_some(),
            "references and foreign source ownership never enter this cascade"
        );
    }
    for id in &expected {
        assert!(
            view.read_exact_entity_state(execution, *id, |_, _| ())
                .unwrap()
                .is_none(),
            "each exact original generation is retired by the one candidate"
        );
    }
    owned_routes::restore_without_owner(
        execution,
        &runtime,
        &installed,
        &owner,
        &selected_meanings[0],
        retired.commit.commit_id,
    );
    for meaning in selected_meanings {
        assert!(
            runtime
                .readmit_authored_output_retirement_successor(
                    execution, &installed, &basis, &meaning
                )
                .unwrap()
                .is_some(),
            "each original route retains its distinct retirement successor"
        );
    }

    for cold in [true, false] {
        restoration::check(
            execution, &runtime, &installed, &owner, &retired, source, &expected, depth, cold,
        );
    }
}

#[test]
fn authored_source_and_separate_realization_publication_retire_atomically() {
    require_atomic_retirement(1);
}

#[test]
fn parent_child_and_leaf_publications_retire_atomically() {
    require_atomic_retirement(3);
}
