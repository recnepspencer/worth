//! Concrete acknowledged subjects authorize their original canonical cutover.
use super::medium_binding::SignalCommittedOwnerEffect;
use super::*;
use crate::branch::owner_services::{
    SignalBranchCellState, SignalBranchExecutionCell, SignalDurableBranchHandle,
    SignalOwnerOperationAdmission,
};
use crate::branch::{AdmittedSignalBranchBasis, SignalBranchForkOutcome};
use crate::logic::transaction::{BranchState, TransactionResult};

pub(super) enum CutoverOwnerEffect {
    Advanced(Box<TransactionResult>),
    Forked(SignalBranchForkOutcome),
}
pub(super) struct CommittedSignalCutover<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) old_state: Option<Box<BranchState<D, I, T>>>,
    pub(super) effect: CutoverOwnerEffect,
    pub(super) branch: SignalDurableBranchHandle,
    pub(super) evidence: crate::durability::SignalCommittedAdvanceEvidence,
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn commit_effect<'a>(
        &self,
        owner: &'a Arc<SignalOwner<D, I, T>>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        cell: Option<Arc<SignalBranchExecutionCell<SignalBranchCellState<D, I, T>>>>,
        proof: SignalCommittedOwnerEffect<D, I, T>,
        retention: SignalBranchAdmissionReservation,
        location: SignalAdvanceLocationReservation,
    ) -> CommittedSignalCutover<D, I, T> {
        let (old_state, effect, basis, record) = match proof {
            SignalCommittedOwnerEffect::Advanced(proof) => {
                let cell = cell.expect("advanced original retains its actual live target");
                let (old, transaction, observation, record) =
                    cell.commit_durable_advance(admission, proof);
                let basis = owner.admit_canonical_basis(
                    observation,
                    cell.branch_id(),
                    cell.incarnation().get(),
                    retention.into_one(),
                );
                (
                    Some(old),
                    CutoverOwnerEffect::Advanced(transaction),
                    basis,
                    record,
                )
            }
            SignalCommittedOwnerEffect::Forked(proof) => {
                let (fork, basis, record) = proof.consume(|subject| {
                    let (mut prepared, record) = subject.into_original();
                    assert_eq!(
                        record.artifact().descriptor().branch().ordinal(),
                        prepared.branch().id.0
                    );
                    prepared
                        .state_mut()
                        .install_durable_coordinates(record.source_point());
                    let installed = prepared.install(owner, admission).expect(
                        "original admission and destination reservation remain held through ACK",
                    );
                    let incarnation = installed.cell().incarnation().get();
                    let (branch, observation) = installed.into_handoff_parts();
                    let basis: AdmittedSignalBranchBasis = owner.admit_canonical_basis(
                        observation,
                        branch.id,
                        incarnation,
                        retention.into_one(),
                    );
                    let retirement = owner
                        .issue_managed_branch_reference_with_admission(admission, &basis)
                        .expect("the same operation installed this exact fork target");
                    let fork =
                        SignalBranchForkOutcome::owner_issued(branch, basis.clone(), retirement);
                    (fork, basis, record)
                });
                (None, CutoverOwnerEffect::Forked(fork), basis, record)
            }
        };
        let evidence = self
            .source
            .lock()
            .expect("source index has no caller code")
            .record_committed_advance(record, location);
        let branch = SignalDurableBranchHandle::from_owner(
            basis,
            evidence.source_point(),
            self.medium.binding().incarnation().fence(),
        );
        CommittedSignalCutover {
            old_state,
            effect,
            branch,
            evidence,
        }
    }
}
