//! #388: a Native effect names its World commit only through the protected
//! image's own open, history head and admitted occurrence, joined by key.
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RelationalAssociatedEffectSubject as Subject;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::mvcc::{RelationalCallerAssociation, RelationalPublicationOutcome};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_world::facade::*;

fn discover(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    receipt: &RelationalCommitReceipt,
) -> Option<Subject> {
    native
        .durability_authority()
        .discover_associated_effect(execution, receipt)
        .unwrap()
}

/// An ordinary Native commit that replays another effect's association bytes.
fn replay(
    native: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    association: &RelationalCallerAssociation,
) -> RelationalCommitReceipt {
    let (_, basis) = native
        .observe_branch(&native.main_branch_identity())
        .unwrap();
    let mut transaction = native
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(crate::batches::create(7), execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, native)
        .unwrap()
        .bind_caller_association(association.clone())
        .unwrap();
    let candidate = native.prepare_validated_proposal(execution, proposal).unwrap();
    let RelationalPublicationOutcome::Performed(performed) =
        native.publication_port().compare_and_publish(execution, candidate)
    else {
        panic!("the replayed Native commit performs")
    };
    let result = native
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    let receipt = result.semantic_result().receipt().clone();
    native.snapshots().release_snapshot(&result.snapshot).unwrap();
    receipt
}

#[test]
fn a_native_effect_joins_only_its_own_published_world_commit_by_key() {
    let directory = tempfile::tempdir().unwrap();
    let foreign_directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::world_occurrence_readmission::open(directory.path(), &execution);
    let owner = fixture.home.world().owner_identity();
    let bootstrap = fixture.bootstrap_root(&execution);
    let expected = bootstrap.product_branch().clone();
    drop(bootstrap);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        expected,
        crate::batches::create(0),
        &cancellation.token(),
    );
    let inspection = fixture.home.world().inspection_port();
    // Captured while the original is acknowledged but not yet published.
    let open = inspection.observe_program_activation_history(&execution).unwrap();
    let performed = fixture.publish(&execution, prepared, &cancellation.token());
    let published = performed.commit().identity().clone();
    let receipt = performed
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    drop(performed.consume());
    let native = fixture.home.relational().runtime();
    let subject = discover(native, &execution, &receipt).expect("the published effect is associated");
    assert_eq!(
        open.resolve_published_native_commit(owner, &subject).unwrap(),
        None,
        "an Open original has no published World commit"
    );
    drop(open);
    let current = inspection.observe_program_activation_history(&execution).unwrap();
    assert_eq!(
        current.resolve_published_native_commit(owner, &subject).unwrap(),
        Some(published)
    );

    let replayed = replay(native, &execution, subject.association().unwrap());
    let replayed = discover(native, &execution, &replayed).expect("the replay is associated");
    assert_eq!(replayed.association(), subject.association());
    assert_eq!(
        current.resolve_published_native_commit(owner, &replayed).unwrap(),
        None,
        "a replayed association on another Native commit is not that occurrence"
    );

    let later = crate::workload::commit(
        native,
        &execution,
        &native.main_branch_identity().branch_id().clone(),
        crate::batches::create(9),
    )
    .unwrap();
    native.snapshots().release_snapshot(&later.snapshot).unwrap();
    assert!(
        discover(native, &execution, later.semantic_result().receipt()).is_none(),
        "an unassociated Native commit has no World subject"
    );

    let foreign = super::world_occurrence_readmission::open(foreign_directory.path(), &execution);
    assert!(matches!(
        current.resolve_published_native_commit(foreign.home.world().owner_identity(), &subject),
        Err(WorldReadmissionDenial::ForeignOwner)
    ));
    drop(current);
    drop(later);
    super::world_occurrence_readmission::close(foreign.home, &execution);
    super::world_occurrence_readmission::close(fixture.home, &execution);
}
