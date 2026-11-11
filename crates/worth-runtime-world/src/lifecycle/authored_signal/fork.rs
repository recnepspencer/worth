//! Reserve the real projection owner's fork without driving physical publication.
use super::*;
use worth_foundational::facade::CompositeAttemptAssociation;
use worth_proof::AdmittedCompositeSignalPublication;
use worth_signal::facade::{
    branch::ValidatedSignalBranchName,
    durability::{
        SignalCompositeForkBinding, SignalDurableAdvanceTask, SignalDurableBranchHandle,
        SignalOnlyForkPublication,
    },
};
pub enum WorldSignalForkPreparationDenial {
    Signal(Box<SignalDurableAdvanceDenial>),
    Bridge(Box<worth_runtime_bridge::facade::BridgeDurableForkPublicationDenial>),
    BridgeUnavailable,
}
impl std::fmt::Debug for WorldSignalForkPreparationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signal(actual) => f.debug_tuple("Signal").field(actual).finish(),
            Self::Bridge(actual) => f.debug_tuple("Bridge").field(actual).finish(),
            Self::BridgeUnavailable => f.write_str("BridgeUnavailable"),
        }
    }
}
trait PreparedFork: Send + Sync {
    fn commitment(&self) -> [u8; 32];
    fn begin(
        self: Box<Self>,
        request: ExecutionRequest<'_, '_>,
        grant: AdmittedCompositeSignalPublication,
        cancellation: &worth_signal::facade::branch::SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, WorldSignalForkPreparationDenial>;
}
pub(crate) struct PreparedWorldSignalFork {
    original: CompositeAttemptAssociation,
    prepared: Box<dyn PreparedFork>,
    _funding: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for PreparedWorldSignalFork {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("PreparedWorldSignalFork")
            .field("original", &self.original)
            .finish_non_exhaustive()
    }
}
impl PreparedWorldSignalFork {
    pub(crate) fn begin(
        self,
        request: ExecutionRequest<'_, '_>,
        permit: &crate::branch::WorldProductCreationPermit,
        issuer: &InstalledCompositeSignalPublicationOwner,
        cancellation: &worth_signal::facade::branch::SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, WorldSignalForkPreparationDenial> {
        let grant = permit.admit_signal_binding(issuer, &self.original, self.prepared.commitment());
        self.prepared.begin(request, grant, cancellation)
    }
}
struct StandaloneFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    binding: SignalCompositeForkBinding<D, I, T>,
}
impl<D, I, T> PreparedFork for StandaloneFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn commitment(&self) -> [u8; 32] {
        self.binding.commitment()
    }
    fn begin(
        self: Box<Self>,
        request: ExecutionRequest<'_, '_>,
        grant: AdmittedCompositeSignalPublication,
        cancellation: &worth_signal::facade::branch::SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, WorldSignalForkPreparationDenial> {
        self.binding
            .begin(
                request,
                grant,
                Box::new(SignalOnlyForkPublication),
                cancellation,
            )
            .map_err(|cause| WorldSignalForkPreparationDenial::Signal(Box::new(cause)))
    }
}
struct BridgeFork {
    assembly: std::sync::Weak<worth_runtime_bridge::facade::BridgeReadmittedAuthoredAssembly>,
    binding: SignalCompositeForkBinding<(), (), ()>,
}
impl PreparedFork for BridgeFork {
    fn commitment(&self) -> [u8; 32] {
        self.binding.commitment()
    }
    fn begin(
        self: Box<Self>,
        request: ExecutionRequest<'_, '_>,
        grant: AdmittedCompositeSignalPublication,
        cancellation: &worth_signal::facade::branch::SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, WorldSignalForkPreparationDenial> {
        let assembly = self
            .assembly
            .upgrade()
            .ok_or(WorldSignalForkPreparationDenial::BridgeUnavailable)?;
        assembly
            .begin_composite_authored_branch_fork(request, self.binding, grant, cancellation)
            .map_err(|cause| WorldSignalForkPreparationDenial::Bridge(Box::new(cause)))
    }
}
impl<D, I, E, Ctx, T> RuntimeWorldAuthoredSignalAttachment<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn prepare_branch_fork(
        &self,
        request: &ExecutionRequest<'_, '_>,
        source: &SignalDurableBranchHandle,
        target: ValidatedSignalBranchName,
        association: CompositeAttemptAssociation,
    ) -> Result<PreparedWorldSignalFork, WorldSignalForkPreparationDenial> {
        let backing = match &self.projection {
            WorldAuthoredSignalProjectionOwner::SignalOnly => {
                std::mem::size_of::<StandaloneFork<D, I, T>>()
            }
            WorldAuthoredSignalProjectionOwner::Bridge(_) => std::mem::size_of::<BridgeFork>(),
        };
        let bytes = backing
            .checked_add(std::mem::size_of::<PreparedWorldSignalFork>())
            .and_then(|bytes| bytes.checked_add(association.branch().name().len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                WorldSignalForkPreparationDenial::Signal(Box::new(
                    SignalDurableAdvanceDenial::Frame(
                        worth_signal::facade::durability::SignalDurableFrameDenial::SizeLimit,
                    ),
                ))
            })?;
        let funding = request.reserve_retained_memory(bytes).map_err(|cause| {
            WorldSignalForkPreparationDenial::Signal(Box::new(SignalDurableAdvanceDenial::Request(
                cause,
            )))
        })?;
        let prepared: Box<dyn PreparedFork> = match &self.projection {
            WorldAuthoredSignalProjectionOwner::SignalOnly => {
                let mutation = self.services.mutation_port();
                let reservation = mutation
                    .reserve_fork(request, source, target)
                    .map_err(|cause| WorldSignalForkPreparationDenial::Signal(Box::new(cause)))?;
                let binding = mutation
                    .prepare_composite_fork_binding(request, reservation, association.clone())
                    .map_err(|cause| WorldSignalForkPreparationDenial::Signal(Box::new(cause)))?;
                Box::new(StandaloneFork { binding })
            }
            WorldAuthoredSignalProjectionOwner::Bridge(owner) => {
                let assembly = owner
                    .upgrade()
                    .ok_or(WorldSignalForkPreparationDenial::BridgeUnavailable)?;
                let mutation = assembly.signal_owner().mutation_port();
                let reservation = mutation
                    .reserve_fork(request, source, target)
                    .map_err(|cause| WorldSignalForkPreparationDenial::Signal(Box::new(cause)))?;
                let binding = mutation
                    .prepare_composite_fork_binding(request, reservation, association.clone())
                    .map_err(|cause| WorldSignalForkPreparationDenial::Signal(Box::new(cause)))?;
                Box::new(BridgeFork {
                    assembly: owner.clone(),
                    binding,
                })
            }
        };
        Ok(PreparedWorldSignalFork {
            original: association,
            prepared,
            _funding: funding,
        })
    }
}
