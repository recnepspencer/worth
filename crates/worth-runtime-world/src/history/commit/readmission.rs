//! Recovered commits use the actual acting owners, never live execution results.
use super::*;
use crate::history::RecoveredWorldOwnerEvidenceRef;
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::durability::RelationalRecordedSuccessorRefusal;

#[derive(Debug)]
pub(crate) enum ReadmittedWorldCommitDenial {
    Request(WorkCeilingDenial),
    CapacityOverflow,
    Native(RelationalRecordedSuccessorRefusal),
    Construction(CompositeCommitConstructionDenial),
}
impl CompositeRuntimeWorldCommit {
    pub(crate) fn from_readmitted_publication(
        request: &ExecutionRequest<'_, '_>,
        identity: CompositeCommitIdentity,
        predecessor: &CompositeRuntimeWorldCommit,
        basis: AdmittedCompositeRuntimeWorldBasis,
        publication: CompositePublicationAttemptIdentity,
        owners: RecoveredWorldOwnerEvidenceRef<'_>,
    ) -> Result<Arc<Self>, ReadmittedWorldCommitDenial> {
        use ReadmittedWorldCommitDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let layout = Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(Layout::new::<Self>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let native_bytes = owners.relational().map_or(0, |native| {
            native.effect().subject().receipt().branch_id.0.len()
        });
        let bytes = layout
            .size()
            .checked_add(native_bytes)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::CapacityOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        let mismatch = || Denial::Construction(CompositeCommitConstructionDenial::BasisMismatch);
        let before = predecessor.basis();
        let relational = match owners.relational() {
            Some(native) => {
                if native.predecessor().admission_identity()
                    != before.relational_basis().admission_identity()
                    || native.successor().admission_identity()
                        != basis.relational_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                RelationalComponentEvidence::Published {
                    commit: native
                        .readmitted_commit_identity()
                        .map_err(Denial::Native)?,
                    basis: basis.relational_basis().admission_identity().clone(),
                }
            }
            None => {
                if before.relational_basis().admission_identity()
                    != basis.relational_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                RelationalComponentEvidence::RetainedExact {
                    basis: basis.relational_basis().admission_identity().clone(),
                }
            }
        };
        let signal = match owners.signal() {
            Some(signal) => {
                if signal.basis().admission_identity() != basis.signal_basis().admission_identity()
                    || signal.basis().observation().branch_id()
                        != before.signal_basis().observation().branch_id()
                    || before
                        .signal_basis()
                        .observation()
                        .generation()
                        .get()
                        .checked_add(1)
                        != Some(signal.basis().observation().generation().get())
                {
                    return Err(mismatch());
                }
                use worth_signal::facade::durability::SignalAuthoredBranchChange;
                let identity = match signal.original().change() {
                    SignalAuthoredBranchChange::Advanced => {
                        CompositeSignalPublicationIdentity::Advanced(
                            signal.basis().admission_identity().clone(),
                        )
                    }
                    SignalAuthoredBranchChange::Created { .. }
                    | SignalAuthoredBranchChange::SnapshotCaptured
                    | SignalAuthoredBranchChange::Restored
                    | SignalAuthoredBranchChange::Retired => return Err(mismatch()),
                };
                SignalComponentEvidence::Published(identity)
            }
            None => {
                if before.signal_basis().admission_identity()
                    != basis.signal_basis().admission_identity()
                    || before.correspondence_basis().admission_identity()
                        != basis.correspondence_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                SignalComponentEvidence::RetainedExact {
                    basis: basis.signal_basis().admission_identity().clone(),
                }
            }
        };
        let result = Self::from_ordinary_components(
            identity,
            predecessor,
            basis,
            publication,
            None,
            memory,
            |_, _| Ok(CompositeComponentEvidence { relational, signal }),
        )
        .map_err(Denial::Construction)?;
        Ok(Arc::new(result))
    }
}
