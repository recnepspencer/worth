//! New authored admission refuses incomplete, foreign, sibling, stale and stopped custody.
use super::*;

#[test]
fn authored_retirement_refuses_partial_missing_and_foreign_members_before_publication() {
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
    let unowned = create_entity(execution, &runtime, "ordinary-unowned-retirement");
    let basis = test_owner_main_basis(&runtime).unwrap();
    let ordinary_roles = crate::mvcc::native_roles_fixture(
        &[(unowned, KindId(1))],
        RequiredOutputRolePosture::Retire,
    );
    assert!(
        runtime
            .readmit_authored_output_retirement(execution, &installed, &basis, &ordinary_roles)
            .unwrap()
            .is_none(),
        "actual unowned births grant no producer succession"
    );
    let roles = retire_roles(ids);
    let partial = runtime
        .readmit_authored_output_retirement(execution, &installed, &basis, &roles[..1])
        .unwrap_err();
    assert!(format!("{partial:?}").contains("omits, repeats or mixes"));
    let wrong_generation = EntityId::new(
        ids[1].partition_id,
        ids[1].local_slot_value(),
        ids[1].generation_value().checked_add(1).unwrap(),
    );
    let mut mixed = roles.clone();
    mixed[1].target = EntityReference::Existing(wrong_generation);
    assert!(runtime
        .readmit_authored_output_retirement(execution, &installed, &basis, &mixed)
        .is_err());
    let repeated = [roles[0].clone(), roles[0].clone()];
    assert!(runtime
        .readmit_authored_output_retirement(execution, &installed, &basis, &repeated)
        .is_err());
    let foreign_owner = worth_proof::InstalledApplicationPublicationOwner::fresh();
    assert!(foreign_owner
        .admit_authored_output_retirement(previous.authority())
        .is_none());
    let mut foreign = runtime_with_test_schema();
    let receipt = foreign
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (_, foreign_installed) = receipt.issue_application_publication_owner();
    assert!(runtime
        .readmit_authored_output_retirement(execution, &foreign_installed, &basis, &roles)
        .is_err());
    // A complete descriptive roster is insufficient without both real deletes.
    let missing = WorkerIntentBatch::new("missing-one-actual-delete")
        .push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent { relation_id: edge },
        )))
        .push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: ids[0] },
        )));
    let denied = publish(
        &runtime,
        execution,
        &installed,
        missing,
        authored(&owner, previous.clone(), roles.clone()),
    )
    .unwrap_err();
    assert!(
        format!("{denied:?}").contains("output kind/lifecycle differs"),
        "{denied:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
    // A missing declared member cannot be hidden by deleting it physically.
    let denied = publish(
        &runtime,
        execution,
        &installed,
        deletes(ids, edge),
        authored(&owner, previous, vec![roles[0].clone()]),
    )
    .unwrap_err();
    assert!(
        format!("{denied:?}").contains("requires every prior live member"),
        "{denied:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
    let side = crate::history::data::BranchId("unowned-retirement-merge".into());
    let (_, fork_source) = runtime
        .observe_fork_source(&crate::history::data::BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(execution, side.clone(), fork_source)
        .unwrap();
    let disjoint =
        create_entity_outcome_on_branch(execution, &runtime, "unrelated-merged-row", side.clone());
    release_test_commit_snapshot(&runtime, &disjoint);
    let merged = test_owner_begin_merge_transaction(
        &runtime,
        crate::history::data::BranchId("main".into()),
        [side],
    )
    .commit(execution, &runtime)
    .unwrap();
    release_test_commit_snapshot(&runtime, &merged);
    let merged_basis = test_owner_main_basis(&runtime).unwrap();
    assert!(runtime.readmit_authored_output_retirement(execution, &installed, &merged_basis, &ordinary_roles).unwrap().is_none(),
        "unrelated merged ancestry cannot turn a proven unowned ordinary Retire into producer ownership");
}

#[test]
fn authored_retirement_refuses_sibling_and_stale_predecessors_and_original_cancellation() {
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
    let (_, source) = runtime
        .observe_fork_source(&crate::history::data::BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(
            execution,
            crate::history::data::BranchId("retirement-sibling".into()),
            source,
        )
        .unwrap();
    let mut sibling = test_owner_begin_transaction_for_branch(
        &runtime,
        crate::history::data::BranchId("retirement-sibling".into()),
    );
    sibling
        .push_batch(
            WorkerIntentBatch::new("sibling-complete-preservation"),
            execution.allocation_policy(),
        )
        .unwrap();
    let proposal = sibling
        .validate(execution, &runtime)
        .unwrap()
        .bind_required_output_publication(&installed, preserve(&owner, previous.clone(), ids, 2))
        .unwrap();
    let candidate = runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the real sibling publication must perform")
    };
    let sibling = runtime.settle_performed_publication(performed).unwrap();
    let sibling_previous = sibling.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &sibling);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let denied = publish(
        &runtime,
        execution,
        &installed,
        deletes(ids, edge),
        authored(&owner, sibling_previous, retire_roles(ids)),
    )
    .unwrap_err();
    assert!(
        format!("{denied:?}").contains("not the publication selected"),
        "{denied:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
    let progressed = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("main-complete-preservation"),
        preserve(&owner, previous.clone(), ids, 3),
    )
    .unwrap();
    let current = progressed.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &progressed);
    let basis = test_owner_main_basis(&runtime).unwrap();
    assert!(
        runtime
            .readmit_authored_output_retirement(
                execution,
                &installed,
                &runtime
                    .observe_branch(
                        &runtime
                            .branch_identity(&crate::history::data::BranchId(
                                "retirement-sibling".into()
                            ))
                            .unwrap()
                    )
                    .unwrap()
                    .1,
                &retire_roles(ids)
            )
            .is_ok(),
        "selected sibling readmission is its own lawful authority"
    );
    let denied = publish(
        &runtime,
        execution,
        &installed,
        deletes(ids, edge),
        authored(&owner, previous, retire_roles(ids)),
    )
    .unwrap_err();
    assert!(
        format!("{denied:?}").contains("not the publication selected"),
        "{denied:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );

    let cancellation = worth_execution::CancellationSource::new();
    let original = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let original_execution = &ExecutionRequest::serial(&original);
    let mut tx = test_owner_begin_transaction_for_main(&runtime);
    tx.push_batch(deletes(ids, edge), original_execution.allocation_policy())
        .unwrap();
    let proposal = tx
        .validate(original_execution, &runtime)
        .unwrap()
        .bind_required_output_publication(&installed, authored(&owner, current, retire_roles(ids)))
        .unwrap();
    cancellation.cancel();
    let denied = runtime
        .prepare_validated_proposal(original_execution, proposal)
        .unwrap_err();
    assert!(
        matches!(
            denied,
            TransactionCommitError::Execution {
                denial: CommitExecutionDenial {
                    kind: CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        ),
        "{denied:?}"
    );
    let denied = runtime
        .readmit_authored_output_retirement(
            original_execution,
            &installed,
            &basis,
            &retire_roles(ids),
        )
        .unwrap_err();
    assert!(
        matches!(
            denied,
            TransactionCommitError::Execution {
                denial: CommitExecutionDenial {
                    kind: CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        ),
        "{denied:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
    let retry = runtime
        .readmit_authored_output_retirement(execution, &installed, &basis, &retire_roles(ids))
        .unwrap()
        .unwrap();
    let mut stale_tx = test_owner_begin_transaction_for_main(&runtime);
    stale_tx
        .push_batch(deletes(ids, edge), execution.allocation_policy())
        .unwrap();
    let stale_proposal = stale_tx
        .validate(execution, &runtime)
        .unwrap()
        .bind_required_output_publication(
            &installed,
            authored(&owner, retry.clone(), retire_roles(ids)),
        )
        .unwrap();
    let unrelated =
        create_entity_outcome(execution, &runtime, "unrelated-before-retirement-prepare");
    release_test_commit_snapshot(&runtime, &unrelated);
    let current_basis = test_owner_main_basis(&runtime).unwrap();
    let denied = runtime
        .prepare_validated_proposal(execution, stale_proposal)
        .unwrap_err();
    assert!(
        matches!(
            denied,
            TransactionCommitError::Conflict {
                error: CommitConflict {
                    class: ConflictClass::StaleValidationBasis { .. },
                    ..
                },
                ..
            }
        ),
        "{denied:?}"
    );
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        current_basis.descriptor()
    );
    assert!(
        runtime
            .readmit_authored_output_retirement(execution, &installed, &basis, &retire_roles(ids))
            .is_err(),
        "old before-image cannot readmit current retirement"
    );
    let retired = publish(
        &runtime,
        execution,
        &installed,
        deletes(ids, edge),
        authored(&owner, retry, retire_roles(ids)),
    )
    .unwrap();
    release_test_commit_snapshot(&runtime, &retired);
}
