//! Prior Retire members stay historical when the current live publication cascades.
use super::*;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;

fn basis(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}
fn role(
    index: usize,
    target: EntityReference,
    posture: RequiredOutputRolePosture,
) -> RequiredOutputRoleTarget {
    RequiredOutputRoleTarget {
        role: format!("output-{index}"),
        entity_name: "node".into(),
        kind: KindId(1),
        posture,
        target,
    }
}
fn publish_rows(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)
        .unwrap()
        .bind_required_output_publication(installed, publication)
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("one complete publication")
    };
    runtime.settle_performed_publication(performed).unwrap()
}
#[test]
fn cascade_retires_and_restores_only_the_complete_live_membership() {
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
    let installed =
        installed.with_source_scope_owned_producers([("source-owned".to_owned(), KindId(1))]);
    let source = targets(&["genuine-source"]);
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(creates("author", &source), execution.allocation_policy())
        .unwrap();
    let born = transaction.commit(execution, &runtime).unwrap();
    let anchor = born.created_entity(&source[0]).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let targets = super::targets(&["realization", "old-cell"]);
    let meaning = RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: "source-owned".into(),
        source_scope: anchor,
        source_partition: worth_foundational::CanonicalDigestId::new([0; 32]),
        source_epoch: worth_foundational::CanonicalDigestId::new([1; 32]),
    };
    let absence = runtime
        .readmit_required_output_route_absence(execution, &installed, &basis(&runtime), &meaning)
        .unwrap();
    let publication = RequiredOutputPublication::Birth {
        admission: owner.admit_required_output("source-owned", [1; 32]),
        absence,
        meaning: meaning.clone(),
        roles: targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                role(
                    index,
                    EntityReference::Created(target.clone()),
                    RequiredOutputRolePosture::Create,
                )
            })
            .collect(),
    };
    let born = publish_rows(
        &runtime,
        execution,
        &installed,
        creates("birth", &targets),
        publication,
    );
    let ids = targets
        .iter()
        .map(|target| born.created_entity(target).unwrap())
        .collect::<Vec<_>>();
    let prior = born.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &born);
    let current = super::targets(&["current-cell"])[0].clone();
    let batch =
        creates("replace-cell", std::slice::from_ref(&current)).push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: ids[1] }),
        ));
    let roles = vec![
        role(
            0,
            EntityReference::Existing(ids[0]),
            RequiredOutputRolePosture::Preserve,
        ),
        role(
            1,
            EntityReference::Existing(ids[1]),
            RequiredOutputRolePosture::Retire,
        ),
        role(
            2,
            EntityReference::Created(current.clone()),
            RequiredOutputRolePosture::Create,
        ),
    ];
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("source-owned", [2; 32]),
            prior.authority(),
        )
        .unwrap();
    let changed = publish_rows(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::Preserve {
            admission,
            predecessor: prior,
            meaning: meaning.clone(),
            roles,
        },
    );
    let cell = changed.created_entity(&current).unwrap();
    release_test_commit_snapshot(&runtime, &changed);
    // Exactly S + {R, Cnew}: three live entities. Cold was already retired.
    let live = [anchor, ids[0], cell];
    let roles = live
        .iter()
        .enumerate()
        .map(|(index, id)| {
            role(
                index,
                EntityReference::Existing(*id),
                RequiredOutputRolePosture::Retire,
            )
        })
        .collect::<Vec<_>>();
    let mut with_old = roles.clone();
    with_old.push(role(
        3,
        EntityReference::Existing(ids[1]),
        RequiredOutputRolePosture::Retire,
    ));
    let failure = runtime
        .readmit_authored_source_retirement(
            execution,
            &installed,
            &basis(&runtime),
            anchor,
            &with_old,
        )
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = failure else {
        panic!("typed old-member refusal")
    };
    assert_eq!(
        error.authored_retirement_refusal(),
        Some(worth_relational::facade::publication::AuthoredRetirementRefusal::ExtraEntity)
    );
    assert_eq!(error.context.affected_records, [RecordRef::Entity(ids[1])]);
    let closure = runtime
        .readmit_authored_source_retirement(execution, &installed, &basis(&runtime), anchor, &roles)
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_authored_output_retirements(closure.authorities())
        .unwrap();
    let batch = live
        .into_iter()
        .fold(WorkerIntentBatch::new("retire-live"), |batch, entity_id| {
            batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id },
            )))
        });
    let retired = publish_rows(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::AuthoredCascadeRetirement {
            admission,
            predecessor: closure,
        },
    );
    assert_eq!(retired.changed_records.len(), 3);
    release_test_commit_snapshot(&runtime, &retired);
    let target = super::targets(&["restored-source"])[0].clone();
    let plan = runtime
        .prepare_authored_source_restoration(
            execution,
            &installed,
            &basis(&runtime),
            retired.commit.commit_id,
            anchor,
            target.clone(),
        )
        .unwrap()
        .unwrap();
    let admission = owner
        .admit_authored_output_retirements(plan.authorities())
        .unwrap();
    let batch = plan
        .extend_batch(
            execution,
            WorkerIntentBatch::new("restore-live").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: target.partition_id,
                    kind_id: target.kind_id,
                    client_key: target.client_key.clone(),
                    fields: fields("genuine-source"),
                }),
            )),
        )
        .unwrap();
    let restored = publish_rows(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::AuthoredCascadeRestoration {
            admission,
            restoration: plan,
        },
    );
    assert_eq!(restored.changed_records.len(), 3);
    assert_ne!(restored.created_entity(&target), Some(anchor));
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&restored.snapshot, &[], execution)
        .unwrap();
    release_test_commit_snapshot(&runtime, &restored);
    let selected = basis(&runtime);
    let view = runtime
        .read_truth()
        .project_observation(&selected.observation())
        .unwrap();
    let mut names = std::collections::BTreeMap::new();
    for record in &restored.changed_records {
        let RecordRef::Entity(id) = record else {
            panic!("row-only fixture")
        };
        let name = view
            .read_exact_entity_state(execution, *id, |_, state| {
                super::restoration::text(state.unwrap(), "name", pinned.symbols())
            })
            .unwrap()
            .unwrap();
        names.insert(name, *id);
    }
    assert_eq!(
        names.keys().map(String::as_str).collect::<Vec<_>>(),
        ["current-cell", "genuine-source", "realization"]
    );
    let meaning = RequiredOutputPublicationMeaning {
        source_scope: names["genuine-source"],
        ..meaning
    };
    let roles = vec![
        role(
            0,
            EntityReference::Existing(names["realization"]),
            RequiredOutputRolePosture::Create,
        ),
        role(
            2,
            EntityReference::Existing(names["current-cell"]),
            RequiredOutputRolePosture::Create,
        ),
    ];
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &selected,
            "source-owned",
            &[2; 32],
            &meaning,
            &roles,
        )
        .unwrap();
}
