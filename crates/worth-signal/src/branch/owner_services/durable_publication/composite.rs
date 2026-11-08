//! One installed composing issuer controls the actual original Signal task.
use super::*;
use crate::branch::owner_services::{SignalDurableBranchSourcePort, SignalDurableSourceDenial};
use crate::durability::SignalCompositeAdvanceBinding;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::{CompositeAttemptAssociation, DurableSignalBranchIdentity};
use worth_proof::{
    AdmittedCompositeSignalPublication, InstalledCompositeSignalPublicationAuthority,
};

pub(super) struct CompositeAdvanceAuthorization {
    pub(super) grant: AdmittedCompositeSignalPublication,
    pub(super) association: CompositeAttemptAssociation,
    pub(super) commitment: [u8; 32],
}

impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(crate) fn attach_composite_owner(
        &self,
        authority: InstalledCompositeSignalPublicationAuthority,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        execution.consult().map_err(Dn::Request)?;
        let mut installed = self
            .composite
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?;
        if let Some(installed) = installed.as_ref() {
            return if installed.same_owner(&authority) {
                Ok(())
            } else {
                Err(Dn::CompositeOwnerAlreadyAttached)
            };
        }
        let table = self
            .tasks
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?;
        if !table.tasks.is_empty() {
            let mut originals =
                ExecutionRequestArrayBuilder::allocate(*execution, table.tasks.len())
                    .map_err(Dn::Backing)?;
            for original in table.tasks.keys() {
                originals.push(*original).map_err(Dn::Backing)?;
            }
            return Err(Dn::CompositeAttachmentPending {
                originals: originals.seal().map_err(Dn::Backing)?,
            });
        }
        *installed = Some(authority);
        Ok(())
    }

    pub(super) fn require_standalone(&self) -> Result<(), SignalDurableAdvanceDenial> {
        if self
            .composite
            .lock()
            .map_err(|_| SignalDurableAdvanceDenial::PublicationStatePoisoned)?
            .is_some()
        {
            return Err(SignalDurableAdvanceDenial::CompositePublicationRequired);
        }
        Ok(())
    }
}

impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The binding is descriptive; only the installed issuer can grant it execution.
    pub fn prepare_composite_binding(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: &crate::branch::AdmittedSignalBranchBasis,
        association: CompositeAttemptAssociation,
    ) -> Result<SignalCompositeAdvanceBinding, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        execution.consult().map_err(Dn::Request)?;
        let state = self.state.upgrade().ok_or(Dn::OwnerUnavailable)?;
        if state
            .composite
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .is_none()
        {
            return Err(Dn::CompositeOwnerNotAttached);
        }
        let owner = state.owner()?;
        if !owner.basis_has_owner_affinity(expected) {
            return Err(Dn::Source(SignalDurableSourceDenial::ForeignBasis));
        }
        let port = SignalDurableBranchSourcePort::new(
            state.owner.clone(),
            state.binding(),
            Arc::clone(&state.vocabulary),
        );
        let branch = DurableSignalBranchIdentity::from_recorded(
            state.binding().owner(),
            expected.branch_id().0,
        );
        let descriptor = port.observe(branch).map_err(Dn::Source)?;
        let actual = port.readmit_descriptor(&descriptor).map_err(Dn::Source)?;
        if actual.basis().admission_identity() != expected.admission_identity() {
            return Err(Dn::Source(SignalDurableSourceDenial::DescriptorMismatch));
        }
        SignalCompositeAdvanceBinding::capture(execution, actual, association).map_err(Dn::Frame)
    }

    pub fn begin_composite_advance<F>(
        &self,
        execution: ExecutionRequest<'_, '_>,
        binding: SignalCompositeAdvanceBinding,
        grant: AdmittedCompositeSignalPublication,
        runtime_ctx: &mut Ctx,
        apply: F,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial>
    where
        F: FnOnce(
            &mut crate::logic::transaction::SignalTransaction<'_, D, I, E, Ctx, T>,
        ) -> Result<(), crate::data::error::SignalError>,
    {
        let (expected, association, commitment) = binding.into_parts();
        self.begin_prepared(
            execution,
            &expected,
            runtime_ctx,
            apply,
            None,
            Some(CompositeAdvanceAuthorization {
                grant,
                association,
                commitment,
            }),
        )
    }
}

impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Resume the exact retained task; never allocate an original or rerun its body.
    pub fn resume_composite_original(
        &self,
        original: SignalBranchAdvanceIdentity,
        grant: AdmittedCompositeSignalPublication,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        let state = self.state.upgrade().ok_or(Dn::OwnerUnavailable)?;
        let authority = state
            .composite
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .clone()
            .ok_or(Dn::CompositeOwnerNotAttached)?;
        let task = state.task(original)?;
        let slot = task.try_lock().map_err(|denial| match denial {
            std::sync::TryLockError::WouldBlock => Dn::TaskDriving { original },
            std::sync::TryLockError::Poisoned(_) => Dn::PublicationStatePoisoned,
        })?;
        let retained = slot
            .composite_grant
            .as_ref()
            .ok_or(Dn::CompositePublicationRequired)?;
        authority
            .admit(&grant, &retained.commitment())
            .map_err(Dn::CompositeAdmission)?;
        Ok(state.handle(original))
    }
}
