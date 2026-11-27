//! Every versioned merge member refusal preserves planning, retry and execute meaning.
use super::{
    lineage_event_matrix::FAILURES,
    merge_member_targets::Subject,
    merge_publications::request,
    merge_source::{assert_merge_records, MergeRecipe},
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;

#[test]
fn residency_failure_matrix_merge_planning_preserves_versioned_footprints() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    // No derived index: a recovered indexed runtime refuses Native merge commit
    // until explicit index recovery, which merge_index_reads owns.
    let plain = MergeRecipe::new(execution, false, false);
    let symbols = MergeRecipe::new(execution, true, false);
    for subject in Subject::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = if matches!(subject, Subject::Symbols) {
                symbols.reopen(execution)
            } else {
                plain.reopen(execution)
            };
            let targets = subject.targets(&fixture);
            assert!(
                !targets.is_empty(),
                "each cell injects actual named members"
            );
            fixture.source.failures.lock().unwrap().extend(
                targets
                    .iter()
                    .map(|member| (ColdRead::Member(*member), cause)),
            );
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = fixture.medium.offered().len();
            let reservations = fixture.runtime.patch_position_reservation_counters();
            let prepared = fixture
                .runtime
                .prepare_merge_execution(execution, request(&fixture.runtime));
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(fixture.medium.offered().len(), offered);
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            let accepted = if let Some(expected) = subject.expected(&fixture, cause) {
                assert_eq!(prepared.unwrap_err(), expected, "{subject:?} x {cause:?}");
                fixture.source.failures.lock().unwrap().clear();
                let failed_reads = || {
                    fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|member| targets.contains(member))
                        .count()
                };
                let before = failed_reads();
                if sticky {
                    assert_eq!(
                        fixture
                            .runtime
                            .prepare_merge_execution(execution, request(&fixture.runtime))
                            .unwrap_err(),
                        expected
                    );
                    assert_eq!(failed_reads(), before);
                    None
                } else {
                    let accepted = fixture
                        .runtime
                        .prepare_merge_execution(execution, request(&fixture.runtime))
                        .unwrap();
                    assert!(failed_reads() > before);
                    Some(accepted)
                }
            } else {
                assert!(
                    !fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|member| targets.contains(member)),
                    "unconsulted {subject:?}"
                );
                Some(prepared.unwrap_or_else(|failure| {
                    panic!("current merge touched {subject:?}: {failure:?}")
                }))
            };
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            if let Some(accepted) = accepted {
                let reads = fixture.source.member_reads.lock().unwrap().len();
                let commits = fixture.source.commit_reads.lock().unwrap().len();
                let outcome = fixture
                    .runtime
                    .execute_prepared_merge(execution, accepted)
                    .unwrap();
                assert_eq!(
                    fixture.source.member_reads.lock().unwrap().len(),
                    reads,
                    "combined execution reuses admitted planning members"
                );
                assert_eq!(
                    fixture.source.commit_reads.lock().unwrap().len(),
                    commits,
                    "execution reuses admitted planning envelopes"
                );
                assert_eq!(outcome.structural_summary.adopted_source_record_count, 1);
                fixture.source.failures.lock().unwrap().clear();
                let actual = fixture
                    .runtime
                    .read_truth()
                    .read_snapshot(&outcome.commit.snapshot, execution)
                    .unwrap()
                    .unwrap();
                assert_merge_records(
                    actual.entities(),
                    &fixture.expected,
                    fixture.expected_merge_version,
                    outcome.commit.version_id,
                );
            }
        }
    }
}

#[test]
fn stale_unnamed_naming_root_cannot_suppress_cold_named_lineage_merge_refusal() {
    use super::bounded_naming_fixture::{create_named_entities, delete_entity, observe_branch};
    use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
    use super::naming_branch_root_and_dissolve::fork_retained_at;
    use super::naming_historical_fork_integrity::durable_mixed_runtime;
    use std::collections::BTreeSet;
    use worth_execution::ExecutionRequest;
    use worth_relational::facade::durability::{
        RecoveryVerificationMode, RelationalDurableCheckpointSource,
        RelationalDurableRecoverySource, RelationalNativeCheckpoint,
    };
    use worth_relational::facade::history::BranchId;
    use worth_relational::facade::identity::{KindId, PartitionId};
    use worth_relational::facade::merge::{
        MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent, MergePlanningError,
    };
    use worth_relational::facade::mvcc::RelationalTransactionIntent;
    use worth_relational::facade::storage::{
        RelationalStorageUnavailable, RelationalStorageUnavailableCause as Cause,
    };
    use worth_relational::facade::transactions::{
        CreateIntent, EntitySpec, MutationIntent, RelationalExecutionDenialCause, WorkerIntentBatch,
    };
    use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
    use worth_testkit_relational::cold_recovery::capture_external_checkpoint;
    use worth_testkit_relational::durable_medium::TestDurableMedium;
    use worth_testkit_relational::durable_publication::field_patch;

    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    let commit_unnamed = |branch: &BranchId, key: &str| {
        let obs = observe_branch(&producer, branch);
        let mut tx = producer
            .begin_branch_transaction(&obs, RelationalTransactionIntent::ordinary())
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(3),
                    client_key: key.into(),
                    fields: field_patch("name", "name", key),
                },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
        let outcome = tx.commit(execution, &producer).unwrap();
        release_test_commit_snapshot(&producer, &outcome);
    };

    // Establish merge base on `main`, fork `feature`, and publish an ordinary
    // unnamed record so `feature` has a valid unnamed `BranchNamingRoot` (tags 8/9 only).
    commit_unnamed(&main, "stale-root-main-base");
    let feature = BranchId("stale-root-feature".into());
    fork_retained_at(
        execution,
        &producer,
        &observe_branch(&producer, &main),
        &feature,
    )
    .unwrap();
    commit_unnamed(&feature, "stale-root-feature-unnamed");
    let (ck_unnamed, _) = capture_external_checkpoint(execution, &producer);

    // Publish a post-base named entity on `feature` and retire it so no live
    // named record remains to adopt; only the cold lineage event records it.
    let (_, entities, _) =
        create_named_entities(execution, &producer, &feature, &["stale-root-born".into()]);
    delete_entity(execution, &producer, &feature, entities[0]);
    let (ck_after, src_after) = capture_external_checkpoint(execution, &producer);
    let post_event_members: BTreeSet<_> = super::selected_directory::read_native(&ck_after)
        .lineage_events
        .iter()
        .map(|page| page.member)
        .collect();
    assert!(!post_event_members.is_empty());

    // Substitute `feature`'s earlier unnamed BranchNamingRoot into `ck_after`
    // while keeping `ck_after`'s intact lineage event pages.
    let extract_feature_root = |ck: &RelationalNativeCheckpoint| -> Vec<u8> {
        let payload = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
        let naming = field(field(payload, "manifest"), "naming");
        rows(field(naming, "branches"))
            .into_iter()
            .find(|entry| {
                rmp_serde::from_slice::<BranchId>(field(entry, "branch")).unwrap() == feature
            })
            .unwrap()
            .to_vec()
    };
    let stale_feature_root = extract_feature_root(&ck_unnamed);
    let after_payload = ck_after.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let after_manifest = field(after_payload, "manifest");
    let after_naming = field(after_manifest, "naming");
    let replaced_branches: Vec<Vec<u8>> = rows(field(after_naming, "branches"))
        .into_iter()
        .map(|entry| {
            if rmp_serde::from_slice::<BranchId>(field(entry, "branch")).unwrap() == feature {
                stale_feature_root.clone()
            } else {
                entry.to_vec()
            }
        })
        .collect();
    let branch_slices: Vec<&[u8]> = replaced_branches.iter().map(Vec::as_slice).collect();
    let tampered_naming = replace_field(after_naming, "branches", &array(&branch_slices));
    let tampered_manifest = replace_field(after_manifest, "naming", &tampered_naming);
    let tampered_payload = replace_field(after_payload, "manifest", &tampered_manifest);
    let stale_checkpoint = RelationalNativeCheckpoint::from_untrusted_bytes(
        [CHECKPOINT_MAGIC, &tampered_payload].concat(),
    );
    let combined_source = src_after.with_published_members(ck_unnamed.members());

    let restore_stale = |src: std::sync::Arc<_>| {
        let mut rt = durable_mixed_runtime(&medium);
        let input = RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                ck_after.covered_position().unwrap(),
                ck_after.covered_branch_transitions().unwrap(),
                stale_checkpoint.clone(),
            )),
            Vec::new(),
            Vec::new(),
            src,
        );
        let plan = rt
            .durability()
            .recovery_plan_from_source(
                execution,
                input,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        rt.durability_recovery().recover(execution, plan).unwrap();
        rt
    };

    // Intact cold event pages: stale unnamed naming root MUST NOT suppress NamedLineage.
    let reopened = restore_stale(combined_source.clone());
    let bound = reopened
        .bind_merge_execution_request(MergeExecutionRequest::new(
            main.clone(),
            feature.clone(),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let refusal = reopened
        .prepare_merge_execution(execution, bound)
        .expect_err("stale unnamed naming root must not bypass post-base NamedLineage refusal");
    assert!(
        matches!(refusal, MergeExecutionPreparationError::NamedLineage(ref br, _) if br == &feature),
        "expected NamedLineage({feature:?}), got {refusal:?}"
    );

    // Damaged cold event pages: stale unnamed naming root MUST NOT suppress integrity refusal.
    let damaged_source = combined_source.with_selected_members(ck_after.members());
    let damaged_source = damaged_source.with_published_members(ck_unnamed.members());
    damaged_source.failures.lock().unwrap().extend(
        post_event_members
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::Damaged)),
    );
    let reopened_damaged = restore_stale(damaged_source);
    let bound_damaged = reopened_damaged
        .bind_merge_execution_request(MergeExecutionRequest::new(
            main,
            feature,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let damaged_refusal = reopened_damaged
        .prepare_merge_execution(execution, bound_damaged)
        .expect_err("stale unnamed naming root must not hide damaged cold event page");
    assert!(
        matches!(
            damaged_refusal,
            MergeExecutionPreparationError::Planning(MergePlanningError::Execution(
                RelationalExecutionDenialCause::StorageUnavailable(RelationalStorageUnavailable {
                    cause: Cause::Damaged,
                    ..
                })
            ))
        ),
        "expected StorageUnavailable(Damaged), got {damaged_refusal:?}"
    );
}
