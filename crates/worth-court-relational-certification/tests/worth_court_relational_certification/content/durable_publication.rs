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
