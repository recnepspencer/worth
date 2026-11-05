//! Original Native custody constructs recovered meaning, never live results.
use super::*;
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_relational::facade::durability::{
    RelationalRecordedSuccessorRefusal, RelationalRecoveredPublicationCustody,
};

#[derive(Debug)]
pub(crate) enum ReadmittedWorldCommitDenial {
    Request(WorkCeilingDenial),
    CapacityOverflow,
    Native(RelationalRecordedSuccessorRefusal),
    Construction(CompositeCommitConstructionDenial),
}

impl CompositeRuntimeWorldCommit {
    pub(crate) fn from_readmitted_native_publication(
        request: &ExecutionRequest<'_, '_>,
        identity: CompositeCommitIdentity,
        predecessor: &CompositeRuntimeWorldCommit,
        basis: AdmittedCompositeRuntimeWorldBasis,
        publication: CompositePublicationAttemptIdentity,
        native: &RelationalRecoveredPublicationCustody,
    ) -> Result<Arc<Self>, ReadmittedWorldCommitDenial> {
        use ReadmittedWorldCommitDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let layout = Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(Layout::new::<Self>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let bytes = layout
            .size()
            .checked_add(native.effect().subject().receipt().branch_id.0.len())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::CapacityOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        if native.predecessor().admission_identity()
            != predecessor.basis().relational_basis().admission_identity()
            || native.successor().admission_identity()
                != basis.relational_basis().admission_identity()
            || predecessor.basis().signal_basis().admission_identity()
                != basis.signal_basis().admission_identity()
            || predecessor
                .basis()
                .correspondence_basis()
                .admission_identity()
                != basis.correspondence_basis().admission_identity()
        {
            return Err(Denial::Construction(
                CompositeCommitConstructionDenial::BasisMismatch,
            ));
        }
        let commit = native
            .readmitted_commit_identity()
            .map_err(Denial::Native)?;
        let result = Self::from_ordinary_components(
            identity,
            predecessor,
            basis,
            publication,
            None,
            memory,
            |_, after| {
                Ok(CompositeComponentEvidence {
                    relational: RelationalComponentEvidence::Published {
                        commit,
                        basis: after.relational_basis().admission_identity().clone(),
                    },
                    signal: SignalComponentEvidence::RetainedExact {
                        basis: after.signal_basis().admission_identity().clone(),
                    },
                })
            },
        )
        .map_err(Denial::Construction)?;
        Ok(Arc::new(result))
    }
}
