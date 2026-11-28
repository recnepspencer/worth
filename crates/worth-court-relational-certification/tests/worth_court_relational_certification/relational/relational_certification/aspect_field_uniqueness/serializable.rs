use super::*;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::mvcc::{RelationalPublicationDenial, RelationalPublicationOutcome};

#[test]
fn independently_validated_writers_meet_at_the_commit_head() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_summary_title_commit_boundary_uniqueness();

    let first = stage_summary(&runtime, KindId(1), "first", "contested", "open")
        .validate(request, &runtime)
        .expect("the key is initially unoccupied");
    let second = stage_summary(&runtime, KindId(1), "second", "contested", "closed")
        .validate(request, &runtime)
        .expect("the concurrent proposal sees the same empty head");
    let first = runtime.prepare_validated_proposal(request, first).unwrap();
    let second = runtime.prepare_validated_proposal(request, second).unwrap();
    let publisher = runtime.publication_port();
    let RelationalPublicationOutcome::Performed(first) =
        publisher.compare_and_publish(request, first)
    else {
        panic!("the first proposal must move the branch head");
    };
    let first = runtime.settle_performed_publication(first).unwrap();
    let before = runtime_marker(&runtime);
    let outcome = publisher.compare_and_publish(request, second);
    let RelationalPublicationOutcome::Denied(RelationalPublicationDenial::UniqueFieldConflict(
        conflict,
    )) = outcome
    else {
        panic!("the second proposal must report the conflicting unique key, got {outcome:?}");
    };
    assert!(matches!(conflict.class, ConflictClass::InvariantViolation {
        fields: InvariantViolationFields::UniqueEntityField { entity_kind_id: KindId(1), value, .. },
        ..
    } if value == AspectValue::String(InternedString::Raw("contested".into()))));
    assert_runtime_marker_unchanged(&runtime, before);
    runtime
        .snapshots()
        .release_snapshot(&first.snapshot)
        .unwrap();
}

#[test]
fn different_kinds_can_hold_the_same_aspect_field_value() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = summary_runtime_with_kinds(None, true);
    register_summary_title_index(&runtime, KindId(1));
    register_summary_title_index(&runtime, KindId(17));

    for (kind, name) in [(KindId(17), "other"), (KindId(1), "main")] {
        stage_summary(&runtime, kind, name, "shared", "open")
            .commit(request, &runtime)
            .expect("the matching field on the other kind is outside this unique index");
    }
    let duplicate = stage_summary(&runtime, KindId(17), "third", "shared", "open")
        .commit(request, &runtime)
        .expect_err("the same kind still cannot claim an occupied value");
    assert!(
        matches!(duplicate, TransactionCommitError::Conflict { error, .. }
        if matches!(error.class, ConflictClass::InvariantViolation {
            fields: InvariantViolationFields::UniqueEntityField { entity_kind_id: KindId(17), .. }, ..
        }))
    );
}

#[test]
fn preexisting_duplicates_allow_unrelated_writes_and_repair() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = summary_runtime_with_kinds(None, false);
    let old = commit_entity_with_summary(request, &runtime, "old", "restored", "open").unwrap();
    let second =
        commit_entity_with_summary(request, &runtime, "new", "restored", "open").unwrap();
    commit_entity_with_summary(request, &runtime, "survivor", "restored", "open").unwrap();
    let index = register_summary_title_index(&runtime, KindId(1));
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    let rebuilt = runtime.index_authority().build_for_basis(
        request,
        DerivedIndexBuildRequest {
            source_commit_id: observation.commit_id().unwrap(),
            branch_id: observation.identity().branch_id().clone(),
            index_ids: vec![index],
        },
        &basis,
    );
    assert_eq!(rebuilt.generations.len(), 1, "{rebuilt:?}");
    commit_entity_with_summary(request, &runtime, "unrelated", "different", "open")
        .expect("the transaction never probes the untouched duplicate key");

    let RecordRef::Entity(old_id) = old.changed_records[0] else {
        panic!("the original commit must report its entity");
    };
    let rewrite = |title| {
        WorkerIntentBatch::new("rewrite-duplicate").push(MutationIntent::Entity(
            EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                entity_id: old_id,
                aspect_patch: whole_summary_patch(&summary_contract(), title, "closed"),
            }),
        ))
    };
    let mut unchanged = begin_main_transaction(&runtime);
    unchanged
        .push_batch(rewrite("restored"), request.allocation_policy())
        .unwrap();
    assert_unique_entity_field_conflict(
        unchanged.commit(request, &runtime).unwrap_err(),
        "restored",
    );
    let mut repaired = begin_main_transaction(&runtime);
    repaired
        .push_batch(rewrite("repaired"), request.allocation_policy())
        .unwrap();
    repaired
        .commit(request, &runtime)
        .expect("rewriting one restored holder away frees its original claim");

    let RecordRef::Entity(second_id) = second.changed_records[0] else {
        panic!("the second commit must report its entity");
    };
    let mut removal = begin_main_transaction(&runtime);
    removal
        .push_batch(
            WorkerIntentBatch::new("discard-duplicate").push(MutationIntent::Entity(
                    EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: second_id }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    removal
        .commit(request, &runtime)
        .expect("deleting one of the two remaining duplicate holders repairs the key");
    assert_unique_entity_field_conflict(
        commit_entity_with_summary(request, &runtime, "collision", "restored", "open").unwrap_err(),
        "restored",
    );
}

#[test]
fn an_installed_unique_index_must_have_a_generation_for_the_head() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = summary_runtime_with_kinds(None, false);
    commit_entity_with_summary(request, &runtime, "existing", "before-index", "open").unwrap();
    let before = runtime_marker(&runtime);
    let index = register_summary_title_index(&runtime, KindId(1));

    let error = commit_entity_with_summary(request, &runtime, "new", "new-key", "open")
        .expect_err("an installed definition alone does not certify absence");
    assert!(
        matches!(error, TransactionCommitError::PublicationDenied { denial, .. }
        if matches!(denial, RelationalPublicationDenial::UniqueFieldIndexUnavailable {
            index_id, entity_kind_id: KindId(1)
        } if index_id == index))
    );
    assert_runtime_marker_unchanged(&runtime, before);
}

#[test]
fn a_prepared_proposal_cannot_skip_a_newly_installed_unique_index() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = summary_runtime_with_kinds(None, false);
    commit_entity_with_summary(request, &runtime, "existing", "older-key", "open").unwrap();
    let proposal = stage_summary(&runtime, KindId(1), "first", "before-index", "open")
        .validate(request, &runtime)
        .unwrap();
    let proposal = runtime
        .prepare_validated_proposal(request, proposal)
        .unwrap();
    register_summary_title_index(&runtime, KindId(1));
    let before = runtime_marker(&runtime);
    let outcome = runtime
        .publication_port()
        .compare_and_publish(request, proposal);
    assert!(matches!(
        outcome,
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::StaleIndexDefinitions)
    ));
    assert_runtime_marker_unchanged(&runtime, before);
}

#[test]
fn only_one_successor_can_claim_the_exact_retiring_holders_key() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = runtime_with_summary_title_commit_boundary_uniqueness();
    let holder = commit_entity_with_summary(request, &runtime, "holder", "held", "open").unwrap();
    let unrelated =
        commit_entity_with_summary(request, &runtime, "other", "other", "open").unwrap();
    let retire = |commit: &CommitResult| {
        let RecordRef::Entity(entity_id) = commit.changed_records[0] else {
            panic!("creation reports the created entity");
        };
        MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
            entity_id,
        }))
    };
    let try_claim = |batch| {
        let mut txn = begin_main_transaction(&runtime);
        txn.push_batch(batch, request.allocation_policy()).unwrap();
        txn.commit(request, &runtime)
    };
    let claim = WorkerIntentBatch::new("transfer-key").push(create_summary(
        KindId(1),
        "successor",
        "held",
        "open",
    ));
    let before = runtime_marker(&runtime);
    assert_unique_entity_field_conflict(
        try_claim(claim.clone().push(retire(&unrelated))).unwrap_err(),
        "held",
    );
    assert_unique_entity_field_conflict(
        try_claim(claim.clone().push(retire(&holder)).push(create_summary(
            KindId(1),
            "second-successor",
            "held",
            "open",
        )))
        .unwrap_err(),
        "held",
    );
    assert_runtime_marker_unchanged(&runtime, before);
    try_claim(claim.push(retire(&holder))).expect("one exact successor can claim the retiring key");
}

#[test]
fn clearing_an_optional_field_frees_the_value_for_the_same_commit() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let runtime = summary_runtime_with_kinds(None, false);
    register_summary_index(&runtime, KindId(1), "status");
    let created =
        commit_entity_with_summary(request, &runtime, "holder", "original", "held").unwrap();
    let RecordRef::Entity(entity_id) = created.changed_records[0] else {
        panic!("creation reports the created entity");
    };
    let aspect_patch =
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
            basis: PortableAspectContractBasis::from_contract(&summary_contract()),
            selected_fields: vec![field_key("status")],
            field_sets: vec![],
            field_clears: vec![field_key("status")],
        }]);
    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("optional-unique-transfer")
            .push(create_summary(KindId(1), "successor", "new", "held"))
            .push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id,
                    aspect_patch,
                }),
            )),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, &runtime)
        .expect("a cleared holder no longer claims its previous value");
}
