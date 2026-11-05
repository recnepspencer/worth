//! Both retirement readmission checkpoints retain the original cancellation.
use super::*;
use crate::authority::commit::pipeline::{stage_observation as observation, CommitStage};

#[test]
fn authored_retirement_readmission_checkpoints_keep_original_cancellation_and_head() {
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
    release_test_commit_snapshot(&runtime, &born);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    let original = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let original_execution = &ExecutionRequest::serial(&original);
    let cancel = cancellation.clone();
    let denied = observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Admission {
                cancel.cancel();
            }
        }),
        || {
            runtime.readmit_authored_output_retirement(
                original_execution,
                &installed,
                &basis,
                &retire_roles(ids),
            )
        },
    )
    .unwrap_err();
    assert_cancelled(denied);
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
    let predecessor = runtime
        .readmit_authored_output_retirement(execution, &installed, &basis, &retire_roles(ids))
        .unwrap()
        .unwrap();
    let retired = publish(
        &runtime,
        execution,
        &installed,
        deletes(ids, edge),
        authored(&owner, predecessor, retire_roles(ids)),
    )
    .unwrap();
    let publication = retired.required_output_publication(&installed).unwrap();
    let meaning = &publication
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning;
    release_test_commit_snapshot(&runtime, &retired);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    let original = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let original_execution = &ExecutionRequest::serial(&original);
    let cancel = cancellation.clone();
    let denied = observation::with_observer(
        Box::new(move |stage| {
            if stage == CommitStage::Admission {
                cancel.cancel();
            }
        }),
        || {
            runtime.readmit_authored_output_retirement_successor(
                original_execution,
                &installed,
                &basis,
                meaning,
            )
        },
    )
    .unwrap_err();
    assert_cancelled(denied);
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
    let successor = runtime
        .readmit_authored_output_retirement_successor(execution, &installed, &basis, meaning)
        .unwrap()
        .unwrap();
    assert_eq!(successor.envelope.commit, publication.envelope.commit);
    assert_eq!(
        test_owner_main_basis(&runtime).unwrap().descriptor(),
        basis.descriptor()
    );
}

fn assert_cancelled(denied: TransactionCommitError) {
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
}

#[test]
fn cascade_refuses_tampered_carried_origin_and_membership_records() {
    for change_origin in [true, false] {
        let request = crate::test_execution_policy::serial_request();
        let execution = &ExecutionRequest::serial(&request);
        let mut runtime = runtime_with_test_schema();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(crate::schema::data::RelationalSchemaRegistry::new())
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let installed =
            installed.with_source_scope_owned_producers([("source-owned".to_owned(), KindId(1))]);
        let anchor = create_entity(execution, &runtime, "genuine-authored-anchor");
        let (batch, targets) = creates(["owned-realization", "owned-cell"]);
        let mut birth = native_birth_fixture(
            owner.admit_required_output("source-owned", [1; 32]),
            targets.to_vec(),
        );
        if let RequiredOutputPublication::Birth { meaning, .. } = &mut birth {
            meaning.source_scope = anchor;
        }
        let born = publish(&runtime, execution, &installed, batch, birth).unwrap();
        let ids = targets.map(|target| born.created_entity(&target).unwrap());
        release_test_commit_snapshot(&runtime, &born);
        let before = test_owner_main_basis(&runtime).unwrap();
        let roles = crate::mvcc::native_roles_fixture(
            &[anchor, ids[0], ids[1]].map(|id| (id, KindId(1))),
            RequiredOutputRolePosture::Retire,
        );
        let mut selected = runtime
            .readmit_authored_source_retirement(execution, &installed, &before, anchor, &roles)
            .unwrap()
            .unwrap();
        let mut tampered: crate::history::data::CanonicalRequiredOutputOrigin =
            rmp_serde::from_slice(
                &rmp_serde::to_vec(selected.publications[0].origin.as_ref()).unwrap(),
            )
            .unwrap();
        let corrupt_subject = if change_origin {
            anchor
        } else {
            tampered.members[0].id
        };
        if change_origin {
            tampered.dependency_identity = worth_foundational::CanonicalDigestId::new([99; 32]);
        } else {
            tampered.members[0].entity_name = "invented-member-family".into();
        }
        selected.publications[0].origin = std::sync::Arc::new(tampered);
        let admission = owner
            .admit_authored_output_retirements(selected.authorities())
            .unwrap();
        let batch = [anchor, ids[0], ids[1]].into_iter().fold(
            WorkerIntentBatch::new("tampered-cascade"),
            |batch, entity_id| {
                batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
                    DeleteEntityIntent { entity_id },
                )))
            },
        );
        let failure = publish(
            &runtime,
            execution,
            &installed,
            batch,
            RequiredOutputPublication::AuthoredCascadeRetirement {
                admission,
                predecessor: selected,
            },
        )
        .unwrap_err();
        let TransactionCommitError::Publication { error, .. } = failure else {
            panic!("typed corruption refusal required")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(crate::publication::data::AuthoredRetirementRefusal::CorruptPublication)
        );
        assert_eq!(
            error.context.affected_records,
            [RecordRef::Entity(corrupt_subject)]
        );
        assert_eq!(
            test_owner_main_basis(&runtime).unwrap().descriptor(),
            before.descriptor()
        );
    }
}
