//! Native content uses the existing durable-before-live publication boundary.
use super::{
    fixture::grid,
    original,
    transaction::{batch, fixture_builder, FORMAT},
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, durability::*, mvcc::*,
    publication::PatchStreamPosition, storage::RelationalStorageUnavailableCause as Cause,
    transactions::*,
};

#[derive(Debug, Default)]
struct RefusingMedium(AtomicUsize);
#[derive(Debug)]
struct Attachment(Arc<RefusingMedium>);
impl RelationalDurableMedium for Attachment {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.0.clone()
    }
    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        Err(RelationalDurableTransitionReadFailure::Missing(sequence))
    }
    fn publish(&self, _: &RelationalCanonicalCommitArtifact) -> DurablePublicationFate<String> {
        self.0 .0.fetch_add(1, Ordering::SeqCst);
        DurablePublicationFate::ProvenNoEffect {
            detail: "test medium certifies no publication effect".into(),
        }
    }
    fn publish_transition(
        &self,
        _: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.0 .0.fetch_add(1, Ordering::SeqCst);
        DurablePublicationFate::ProvenNoEffect {
            detail: "test medium certifies no transition effect".into(),
        }
    }
}
impl RelationalDurableColdSource for RefusingMedium {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if members.is_empty() {
            Ok(RelationalDurableMemberCustody::new(()))
        } else {
            Err(Cause::SourceUnavailable)
        }
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

#[test]
fn content_medium_refusal_preserves_selected_state_after_complete_native_preparation() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (builder, contract) = fixture_builder();
    let medium = Arc::new(RefusingMedium::default());
    let runtime = builder
        .build_durable(Box::new(Attachment(medium.clone())))
        .unwrap();
    let (before, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let id = transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    transaction
        .push_batch(batch(&contract, id), execution.allocation_policy())
        .unwrap();
    assert!(matches!(
        transaction.commit(&execution, &runtime),
        Err(TransactionCommitError::PublicationDenied {
            denial: RelationalPublicationDenial::DurablePublicationRefused(
                RelationalDurablePublicationRefusal::ProvenNoEffect { .. }
            ),
            ..
        })
    ));
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    assert_eq!(medium.0.load(Ordering::SeqCst), 1);
}

fn large_candidate(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    contract: &worth_foundational::facade::AspectContract,
    samples: u32,
) -> PreparedRelationalCommitCandidate {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let input = super::fixture::Grid {
        rows: 1,
        columns: samples,
        samples: vec![0; samples as usize],
    };
    let id = transaction
        .admit_content(execution, FORMAT, &input)
        .unwrap();
    transaction
        .push_batch(batch(contract, id), execution.allocation_policy())
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

#[test]
fn content_durable_artifact_memory_refuses_before_the_medium_and_selected_commit() {
    use worth_execution::{SerialMemoryBudget, SerialRequest};
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(768 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let medium = Arc::new(RefusingMedium::default());
    let (builder, contract) = fixture_builder();
    let runtime = builder
        .build_durable(Box::new(Attachment(medium.clone())))
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let candidate = large_candidate(&runtime, &execution, &contract, 65_536);
    let refusal = runtime
        .publication_port()
        .compare_and_publish(&execution, candidate);
    assert!(
        matches!(
            refusal,
            RelationalPublicationOutcome::Denied(
                RelationalPublicationDenial::DurablePublicationRefused(
                    RelationalDurablePublicationRefusal::Encoding(
                        RelationalCanonicalCommitArtifactEncodeDenial::Request(
                            RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
                        )
                    )
                )
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(medium.0.load(Ordering::SeqCst), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
    drop(refusal);
    drop(runtime);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn content_durable_artifact_cancellation_after_original_allocation_never_calls_medium() {
    use std::sync::{atomic::AtomicBool, Barrier};
    use worth_execution::{SerialMemoryBudget, SerialRequest};
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let medium = Arc::new(RefusingMedium::default());
    let (builder, contract) = fixture_builder();
    let runtime = builder
        .build_durable(Box::new(Attachment(medium.clone())))
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let candidate = large_candidate(&runtime, &execution, &contract, 1_048_576);
    let prepared_bytes = budget.charged_memory_bytes();
    let done = AtomicBool::new(false);
    let ready = Barrier::new(2);
    let (refusal, observed) = std::thread::scope(|scope| {
        let observer = scope.spawn(|| {
            ready.wait();
            while !done.load(Ordering::Acquire) {
                if budget.charged_memory_bytes() >= prepared_bytes + 8 * 1024 * 1024 {
                    cancel.cancel();
                    return true;
                }
                std::thread::yield_now();
            }
            false
        });
        ready.wait();
        let result = runtime
            .publication_port()
            .compare_and_publish(&execution, candidate);
        done.store(true, Ordering::Release);
        (result, observer.join().unwrap())
    });
    assert!(
        observed,
        "observer saw actual original-funded artifact backing"
    );
    assert!(
        matches!(
            refusal,
            RelationalPublicationOutcome::Denied(
                RelationalPublicationDenial::DurablePublicationRefused(
                    RelationalDurablePublicationRefusal::Encoding(
                        RelationalCanonicalCommitArtifactEncodeDenial::Request(
                            RelationalExecutionDenialCause::Cancelled
                        )
                    )
                )
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(medium.0.load(Ordering::SeqCst), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
    drop(refusal);
    drop(runtime);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn content_durable_artifact_original_work_exhaustion_is_exact_and_pre_effect() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let medium = Arc::new(RefusingMedium::default());
    let (builder, contract) = fixture_builder();
    let runtime = builder
        .build_durable(Box::new(Attachment(medium.clone())))
        .unwrap();
    let before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let candidate = large_candidate(&runtime, &execution, &contract, 65_536);
    let mut refusal = None;
    let mut candidate = Some(candidate);
    let stopped = worth_execution::ExecutionWorkCeiling::new(Some(200)).run_with_checkpoint(
        None,
        Some(&request),
        |_| {
            refusal = Some(
                runtime
                    .publication_port()
                    .compare_and_publish(&execution, candidate.take().unwrap()),
            );
        },
    );
    let ((), report) = stopped
        .expect("the caller receives the typed inner refusal without inventing an outer stop");
    assert!(report.charged_work() <= 200);
    let refusal = refusal.expect("the original accounted publication stage ran");
    assert!(
        matches!(
            refusal,
            RelationalPublicationOutcome::Denied(
                RelationalPublicationDenial::DurablePublicationRefused(
                    RelationalDurablePublicationRefusal::Encoding(
                        RelationalCanonicalCommitArtifactEncodeDenial::Request(
                            RelationalExecutionDenialCause::WorkExhausted
                        )
                    )
                )
            )
        ),
        "{refusal:?}"
    );
    assert_eq!(medium.0.load(Ordering::SeqCst), 0);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        before.descriptor()
    );
    drop(refusal);
    drop(runtime);
    assert_eq!(
        request.memory().unwrap().available_memory_bytes(),
        Some(64 * 1024 * 1024)
    );
}
