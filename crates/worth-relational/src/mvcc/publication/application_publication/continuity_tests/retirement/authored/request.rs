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
