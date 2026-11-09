//! Reserve and capture the original exact fork before any physical publication.
use super::composite::CompositePublicationAuthorization;
use super::*;
use crate::branch::owner_services::{
    SignalBranchMutationPort, SignalDurableBranchHandle, SignalDurableSourceDenial,
    SignalOwnerCancellationToken,
};
use crate::branch::{
    SignalCompositeForkBinding, SignalDurableForkReservation, ValidatedSignalBranchName,
};
use crate::durability::SignalDurableForkPublication;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableSignalBranchIdentity;
use worth_proof::AdmittedCompositeSignalPublication;

impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn reserve_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        name: ValidatedSignalBranchName,
    ) -> Result<SignalDurableForkReservation<D, I, T>, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        execution.consult().map_err(Dn::Request)?;
        let state = self.state.upgrade().ok_or(Dn::OwnerUnavailable)?;
        let owner = Self::admit_fork_owner(&state, expected)?;
        let source_port = crate::branch::owner_services::SignalDurableBranchSourcePort::new(
            state.owner.clone(),
            Arc::downgrade(&state),
            state.binding(),
            Arc::clone(&state.vocabulary),
        );
        let source = source_port
            .readmit_descriptor(expected.descriptor())
            .map_err(Dn::Source)?;
        let memory = SignalBranchMutationPort::<D, I, E, Ctx, T>::new(
            Arc::downgrade(&owner),
            owner.runtime_instance_id(),
        );
        let reservation = memory
            .reserve_fork_exact(name, source.basis())
            .map_err(Dn::Fork)?;
        let branch = DurableSignalBranchIdentity::from_recorded(
            state.binding().owner(),
            reservation.branch().id.0,
        );
        let cell = state
            .source
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .frontiers
            .issue_fork_cell(branch)
            .map_err(Dn::Readmission)?;
        Ok(SignalDurableForkReservation {
            reservation,
            source,
            cell,
        })
    }
    pub fn prepare_composite_fork_binding(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        reservation: SignalDurableForkReservation<D, I, T>,
        association: worth_foundational::facade::CompositeAttemptAssociation,
    ) -> Result<SignalCompositeForkBinding<D, I, T>, SignalDurableAdvanceDenial> {
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
        Self::admit_fork_owner(&state, &reservation.source)?;
        SignalCompositeForkBinding::capture(execution, reservation, association, self.state.clone())
            .map_err(Dn::Frame)
    }
    pub fn begin_fork(
        &self,
        execution: ExecutionRequest<'_, '_>,
        reservation: SignalDurableForkReservation<D, I, T>,
        publication: Box<dyn SignalDurableForkPublication>,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        self.begin_prepared_fork(execution, reservation, publication, cancellation, None)
    }
    fn admit_fork_owner(
        state: &SignalDurablePublicationState<D, I, T>,
        expected: &SignalDurableBranchHandle,
    ) -> Result<Arc<SignalOwner<D, I, T>>, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        if expected.descriptor().owner() != state.binding().owner() {
            return Err(Dn::Source(SignalDurableSourceDenial::ForeignOwner));
        }
        state
            .binding()
            .incarnation()
            .fence()
            .admit(expected.fence())
            .map_err(|cause| Dn::Source(SignalDurableSourceDenial::HomeIncarnation(cause)))?;
        let owner = state.owner()?;
        if !owner.basis_has_owner_affinity(expected.basis()) {
            return Err(Dn::Source(SignalDurableSourceDenial::ForeignBasis));
        }
        Ok(owner)
    }
    fn begin_prepared_fork(
        &self,
        execution: ExecutionRequest<'_, '_>,
        reservation: SignalDurableForkReservation<D, I, T>,
        publication: Box<dyn SignalDurableForkPublication>,
        cancellation: &SignalOwnerCancellationToken,
        authorization: Option<CompositePublicationAuthorization>,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        let state = self.state.upgrade().ok_or(Dn::OwnerUnavailable)?;
        let owner = Self::admit_fork_owner(&state, &reservation.source)?;
        let admission = owner
            .admit()
            .map_err(super::super::mutation_port::map_fork_admission_denial)
            .map_err(Dn::Fork)?;
        let source = reservation.source.descriptor().clone();
        let branches = ReservedSignalBranches::Forked {
            source: reservation.source.basis().owner_branch_id(),
            target: reservation.reservation.branch().id,
        };
        let mut registered = state.register_prepared(&execution, branches, authorization)?;
        let cell = reservation.cell;
        let (prepared, retention) = reservation
            .reservation
            .prepare_for_durable(&owner, &admission, cancellation)
            .map_err(Dn::Fork)?;
        let phase = state.prepare_fork_frames(
            registered.original,
            registered.new_admission,
            source,
            cell,
            prepared,
            retention,
            publication,
            registered.association.take(),
            &execution,
        )?;
        registered.seal(phase)
    }
}

impl<D, I, T> SignalCompositeForkBinding<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Consume this original owner's prepared fork. Forking has no application
    /// mutation closure or context, so its retained binding carries no such lifetime.
    pub fn begin(
        self,
        execution: ExecutionRequest<'_, '_>,
        grant: AdmittedCompositeSignalPublication,
        publication: Box<dyn SignalDurableForkPublication>,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        let state = self
            .publication
            .upgrade()
            .ok_or(SignalDurableAdvanceDenial::OwnerUnavailable)?;
        let port = SignalDurableBranchMutationPort::<D, I, (), (), T>::from_owner(&state);
        port.begin_prepared_fork(
            execution,
            self.reservation,
            publication,
            cancellation,
            Some(CompositePublicationAuthorization {
                grant,
                association: self.association,
                commitment: self.commitment,
            }),
        )
    }
}
