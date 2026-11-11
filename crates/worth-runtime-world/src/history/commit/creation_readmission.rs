//! Cold branch commits consume actual fork admissions, never live owner results.
use super::*;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::ReadmittedRelationalAssociatedFork;
use worth_signal::facade::durability::{SignalAuthoredBranchChange, SignalReadmittedSourcePoint};

impl CompositeRuntimeWorldCommit {
    pub(crate) fn from_readmitted_branch_creation(
        request: &ExecutionRequest<'_, '_>,
        identity: CompositeCommitIdentity,
        predecessor: &Self,
        basis: AdmittedCompositeRuntimeWorldBasis,
        attempt: CompositePublicationAttemptIdentity,
        native: Option<&ReadmittedRelationalAssociatedFork>,
        signal: Option<&SignalReadmittedSourcePoint>,
    ) -> Result<Arc<Self>, ReadmittedWorldCommitDenial> {
        use ReadmittedWorldCommitDenial as Denial;
        let mismatch = || Denial::Construction(CompositeCommitConstructionDenial::BasisMismatch);
        if native.is_none() && signal.is_none() {
            return Err(mismatch());
        }
        let relational = match native {
            Some(actual) => {
                if actual.source_basis().descriptor()
                    != predecessor.basis().relational_basis().descriptor()
                    || actual.target_basis().admission_identity()
                        != basis.relational_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                RelationalComponentEvidence::Forked {
                    target: actual.target_basis().identity().clone(),
                    basis: basis.relational_basis().admission_identity().clone(),
                }
            }
            None => {
                if predecessor.basis().relational_basis().admission_identity()
                    != basis.relational_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                RelationalComponentEvidence::RetainedExact {
                    basis: basis.relational_basis().admission_identity().clone(),
                }
            }
        };
        let signal = match signal {
            Some(actual) => {
                if actual.basis().admission_identity() != basis.signal_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                match actual.original().change() {
                    SignalAuthoredBranchChange::Created {
                        parent: Some(_), ..
                    } => {}
                    SignalAuthoredBranchChange::Created { parent: None, .. }
                    | SignalAuthoredBranchChange::Advanced
                    | SignalAuthoredBranchChange::SnapshotCaptured
                    | SignalAuthoredBranchChange::Restored
                    | SignalAuthoredBranchChange::Retired => return Err(mismatch()),
                }
                SignalComponentEvidence::Published(CompositeSignalPublicationIdentity::Forked(
                    basis.signal_basis().admission_identity().clone(),
                ))
            }
            None => {
                if predecessor.basis().signal_basis().admission_identity()
                    != basis.signal_basis().admission_identity()
                    || predecessor
                        .basis()
                        .correspondence_basis()
                        .admission_identity()
                        != basis.correspondence_basis().admission_identity()
                {
                    return Err(mismatch());
                }
                SignalComponentEvidence::RetainedExact {
                    basis: basis.signal_basis().admission_identity().clone(),
                }
            }
        };
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let names = native.map_or(0, |actual| {
            actual.target_basis().descriptor().branch_id().0.len()
        });
        let bytes = layout
            .size()
            .checked_add(names)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::CapacityOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        let commit = Self::from_ordinary_components(
            identity,
            predecessor,
            basis,
            attempt,
            None,
            memory,
            |_, _| Ok(CompositeComponentEvidence { relational, signal }),
        )
        .map_err(Denial::Construction)?;
        Ok(Arc::new(commit))
    }
}
