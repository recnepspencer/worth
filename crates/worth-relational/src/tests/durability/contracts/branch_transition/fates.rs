use super::*;
use crate::branch::{
    RelationalBranchArchiveDenial, RelationalBranchDeleteDenial, RelationalBranchDeletionOutcome,
    RelationalBranchLifecyclePosture, RelationalForkDenial,
};
use crate::durability::medium::{
    RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject,
};
use crate::facade::mvcc::RelationalTransactionIntent;

#[derive(Clone, Copy, Debug)]
enum Action {
    Fork,
    Archive,
    RequestDeletion,
    Delete,
}

#[derive(Debug)]
enum TransitionDenial {
    Fork(RelationalForkDenial),
    Archive(RelationalBranchArchiveDenial),
    Delete(RelationalBranchDeleteDenial),
}

impl TransitionDenial {
    fn publication_refused(&self) -> bool {
        matches!(
            self,
            Self::Fork(RelationalForkDenial::DurablePublicationRefused(_))
                | Self::Archive(RelationalBranchArchiveDenial::DurablePublicationRefused(_))
                | Self::Delete(RelationalBranchDeleteDenial::DurablePublicationRefused(_))
        )
    }
}

fn normalize<D>(stop: Stop<D>, denial: fn(D) -> TransitionDenial) -> Stop<TransitionDenial> {
    match stop {
        Stop::Denied(reason) => Stop::Denied(denial(reason)),
        Stop::RequestStopped(stop) => Stop::RequestStopped(stop),
        Stop::DurabilityUnresolved(fate) => Stop::DurabilityUnresolved(fate),
    }
}

impl Action {
    fn apply(
        self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &mut RelationalRuntime,
    ) -> Result<(), Stop<TransitionDenial>> {
        if matches!(self, Self::Fork) {
            let (_, basis) = runtime
                .observe_fork_source(&BranchId("main".into()))
                .unwrap();
            return runtime
                .fork_branch(execution, BranchId("target".into()), basis)
                .map(|_| ())
                .map_err(|stop| normalize(stop, TransitionDenial::Fork));
        }
        let identity = runtime.branch_identity(&BranchId("target".into())).unwrap();
        match self {
            Self::Archive => runtime
                .archive_branch(execution, &identity)
                .map(|_| ())
                .map_err(|stop| normalize(stop, TransitionDenial::Archive)),
            Self::RequestDeletion => {
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                let transaction = runtime
                    .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                    .unwrap();
                let result = runtime
                    .owner_component_services()
                    .lifecycle_port()
                    .delete_branch(execution, &identity);
                if let Ok(outcome) = &result {
                    assert!(matches!(
                        outcome,
                        RelationalBranchDeletionOutcome::WaitingForActiveOperations(_)
                    ));
                }
                drop(transaction);
                result
                    .map(|_| ())
                    .map_err(|stop| normalize(stop, TransitionDenial::Delete))
            }
            Self::Delete => runtime
                .delete_branch(execution, &identity)
                .map(|_| ())
                .map_err(|stop| normalize(stop, TransitionDenial::Delete)),
            Self::Fork => unreachable!(),
        }
    }
}

// The registry, head index and retention are distinct live consequences.
fn footprint(runtime: &RelationalRuntime) -> String {
    format!(
        "{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{:?}",
        runtime.history.branch_lifecycle_snapshot(),
        runtime.branch_reference_state(&BranchId("target".into())),
        runtime.history().branch_head(&BranchId("target".into())),
        runtime.history.retired_branch_root_count(),
        runtime.history.oldest_branch_head_version(),
        runtime.history.retired_branch_names(),
        artifacts(runtime)
    )
}

fn assert_blocked(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &mut RelationalRuntime,
    fate: &RelationalDurabilityUnresolved,
) {
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    let later = runtime
        .fork_branch(execution, BranchId("later".into()), basis)
        .unwrap_err();
    assert_eq!(later.durability_unresolved(), Some(fate));
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch_create("later"),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let later = transaction.commit(execution, runtime).unwrap_err();
    assert_eq!(later.durability_unresolved_fate(), Some(fate));
}

#[test]
fn every_transition_fate_preserves_the_durable_before_live_boundary() {
    for action in [
        Action::Fork,
        Action::Archive,
        Action::RequestDeletion,
        Action::Delete,
    ] {
        for answer in [
            None,
            Some(ScriptedPublication::ProvenNoEffect),
            Some(ScriptedPublication::IndeterminateWithoutEffect),
            Some(ScriptedPublication::IndeterminateAfterEffect),
        ] {
            let request = crate::test_execution_policy::serial_request();
            let execution = &ExecutionRequest::serial(&request);
            let mut runtime = durable_runtime_with_test_schema();
            let commit = create_entity_outcome(execution, &runtime, "anchor");
            release_test_commit_snapshot(&runtime, &commit);
            if !matches!(action, Action::Fork) {
                fork(execution, &runtime, "target");
            }
            let latest = create_entity_outcome(execution, &runtime, "source-advanced");
            release_test_commit_snapshot(&runtime, &latest);
            let medium = attached_test_medium(&runtime);
            let before = footprint(&runtime);
            let sequence = medium.transition_head().0 + 1;
            if let Some(answer) = answer {
                medium.script_next_transition(answer);
            }
            let result = action.apply(execution, &mut runtime);
            match answer {
                None => {
                    result.unwrap();
                    assert_ne!(footprint(&runtime), before, "{action:?}");
                    assert_eq!(medium.transition_head(), Sequence(sequence));
                    let artifact = Artifact::readmit(
                        &medium.transition_bytes(Sequence(sequence)).unwrap(),
                        execution,
                    )
                    .unwrap();
                    assert_eq!(artifact.sequence(), Sequence(sequence));
                    assert_eq!(artifact.follows(), latest.patch_position());
                }
                Some(ScriptedPublication::ProvenNoEffect) => {
                    let Stop::Denied(denial) = result.unwrap_err() else {
                        panic!("expected pre-effect refusal")
                    };
                    assert!(denial.publication_refused(), "{denial:?}");
                    assert_eq!(footprint(&runtime), before, "{action:?}");
                    assert_eq!(medium.transition_head(), Sequence(sequence - 1));
                    action.apply(execution, &mut runtime).unwrap();
                    assert_eq!(medium.transition_head(), Sequence(sequence));
                }
                Some(_) => {
                    let fate = result.unwrap_err().durability_unresolved().unwrap().clone();
                    let offered = medium.offered_transitions().pop().unwrap();
                    let RelationalDurabilityUnresolvedSubject::BranchTransition {
                        sequence: found,
                        follows,
                        digest,
                    } = fate.subject()
                    else {
                        panic!("expected a transition fate")
                    };
                    assert_eq!(found, Sequence(sequence));
                    assert_eq!(follows, latest.patch_position());
                    assert_eq!(digest, offered.payload_digest());
                    assert_eq!(footprint(&runtime), before, "{action:?}");
                    let offers = (medium.offered().len(), medium.offered_transitions().len());
                    assert_blocked(execution, &mut runtime, &fate);
                    assert_eq!(
                        (medium.offered().len(), medium.offered_transitions().len()),
                        offers
                    );
                    assert_eq!(footprint(&runtime), before, "{action:?}");
                }
            }
            assert_eq!(
                medium.transition_sequences(),
                (1..=medium.transition_head().0)
                    .map(Sequence)
                    .collect::<Vec<_>>()
            );
            let recovered = reopen(execution, &runtime);
            let took_effect = !matches!(
                answer,
                Some(ScriptedPublication::IndeterminateWithoutEffect)
            );
            let cell = recovered.history.branch_cell(&BranchId("target".into()));
            match action {
                Action::Fork => assert_eq!(cell.is_some(), took_effect),
                Action::Archive => assert_eq!(
                    cell.unwrap().lifecycle_posture(),
                    if took_effect {
                        RelationalBranchLifecyclePosture::Archived
                    } else {
                        RelationalBranchLifecyclePosture::Live
                    }
                ),
                Action::RequestDeletion => assert_eq!(
                    cell.unwrap().lifecycle_posture(),
                    if took_effect {
                        RelationalBranchLifecyclePosture::Deleting
                    } else {
                        RelationalBranchLifecyclePosture::Live
                    }
                ),
                Action::Delete => assert_eq!(cell.is_none(), took_effect),
            }
            create_entity(execution, &recovered, "after-resolution");
        }
    }
}

#[test]
fn an_unresolved_commit_also_blocks_branch_transitions_with_its_original_fate() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "anchor");
    let medium = attached_test_medium(&runtime);
    medium.script_next_publication(ScriptedPublication::IndeterminateAfterEffect);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("unresolved"),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let fate = transaction
        .commit(execution, &runtime)
        .unwrap_err()
        .durability_unresolved_fate()
        .unwrap()
        .clone();
    assert_blocked(execution, &mut runtime, &fate);
    assert!(medium.offered_transitions().is_empty());
}
