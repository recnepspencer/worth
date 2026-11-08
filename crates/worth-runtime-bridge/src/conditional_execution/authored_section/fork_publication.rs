//! Fork preparation rebuilds a target projection beside the same Signal original.
use super::{
    branch_projection::{BranchPosture, BranchProjection},
    branch_registry::{BranchRegistry, BranchSlot},
    readmission::{Installation, Program},
    BridgeAuthoredBranchDenial, BridgeAuthoredReadmissionDenial, BridgeAuthoredSection,
    BridgeReadmittedAuthoredAssembly,
};
use crate::conditional_execution::BridgeInstalledConditionalLowering;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::{Arc, Mutex},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{DurableSignalBranchIdentity, SignalBranchAdvanceIdentity};
use worth_signal::facade::branch::SignalOwnerCancellationToken;
use worth_signal::facade::durability::{
    SignalCompositeForkBinding, SignalDurableAdvanceDenial, SignalDurableAdvanceTask,
    SignalDurableBranchHandle, SignalDurableForkPublication, SignalDurableForkReservation,
    SignalDurableProjectionPublication, SignalForkPublicationContext,
};
mod bindings;

#[derive(Debug)]
pub enum BridgeDurableForkPublicationDenial {
    Branch(BridgeAuthoredBranchDenial),
    TargetOccupied { branch: DurableSignalBranchIdentity },
    SourceChanged,
    Readmission(Box<BridgeAuthoredReadmissionDenial>),
    Signal(Box<SignalDurableAdvanceDenial>),
}
type D = BridgeDurableForkPublicationDenial;
enum Phase {
    Preparing,
    Prepared(SignalBranchAdvanceIdentity),
    Committed(SignalBranchAdvanceIdentity),
    Installed,
}
struct Reservation {
    registry: Arc<BranchRegistry>,
    source: Arc<BranchProjection>,
    target: DurableSignalBranchIdentity,
    phase: Phase,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if matches!(self.phase, Phase::Installed) {
            return;
        }
        let mut slots = self
            .registry
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match self.phase {
            Phase::Preparing | Phase::Prepared(_) => {
                slots.remove(&self.target);
            }
            Phase::Committed(original) => {
                *slots
                    .get_mut(&self.target)
                    .expect("original retains its exact reserved slot") =
                    BranchSlot::ReadmissionRequired(original);
            }
            Phase::Installed => unreachable!("installed reservation returns above"),
        }
        drop(slots);
        self.source
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .posture = BranchPosture::Serving;
    }
}
struct Pending {
    reservation: Reservation,
    program: Arc<Program>,
    inherited: Vec<Arc<BridgeInstalledConditionalLowering>>,
    prepared: Option<Arc<BranchProjection>>,
    _memory: Option<ExecutionMemoryReservation>,
    refusal: Arc<Mutex<Option<D>>>,
}
impl BridgeReadmittedAuthoredAssembly {
    pub fn begin_authored_branch_fork(
        &self,
        execution: ExecutionRequest<'_, '_>,
        reservation: SignalDurableForkReservation<(), (), ()>,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, D> {
        let pending = self.prepare_branch_fork(
            &execution,
            reservation.source().descriptor().branch(),
            reservation.target(),
        )?;
        let refusal = Arc::clone(&pending.refusal);
        let result =
            self.signal
                .mutation_port()
                .begin_fork(execution, reservation, pending, cancellation);
        fork_result(result, refusal)
    }
    pub fn begin_composite_authored_branch_fork(
        &self,
        execution: ExecutionRequest<'_, '_>,
        binding: SignalCompositeForkBinding<(), (), ()>,
        grant: worth_proof::AdmittedCompositeSignalPublication,
        cancellation: &SignalOwnerCancellationToken,
    ) -> Result<SignalDurableAdvanceTask, D> {
        let pending = self.prepare_branch_fork(
            &execution,
            binding.source().descriptor().branch(),
            binding.target(),
        )?;
        let refusal = Arc::clone(&pending.refusal);
        let result = self.signal.mutation_port().begin_composite_fork(
            execution,
            binding,
            grant,
            pending,
            cancellation,
        );
        fork_result(result, refusal)
    }
    fn prepare_branch_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        source: DurableSignalBranchIdentity,
        target: DurableSignalBranchIdentity,
    ) -> Result<Box<Pending>, D> {
        let projection = self.branches.projection(source).map_err(D::Branch)?;
        let mut serving = projection.serving(source).map_err(D::Branch)?;
        let memory = execution
            .reserve_retained_memory(
                crate::conditional_execution::retention::sum(&[
                    std::mem::size_of::<Pending>() as u64,
                    crate::conditional_execution::retention::array_charge::<
                        Arc<BridgeInstalledConditionalLowering>,
                    >(serving.lowerings.values().count())
                    .map_err(size_denial)?,
                ])
                .map_err(size_denial)?,
            )
            .map_err(request_denial)?;
        let entry_memory = execution
            .reserve_retained_memory(
                crate::conditional_execution::retention::btree_charge::<
                    DurableSignalBranchIdentity,
                    BranchSlot,
                >(1)
                .map_err(size_denial)?,
            )
            .map_err(request_denial)?;
        let mut slots = self.branches.slots.try_lock().map_err(|cause| {
            D::Branch(match cause {
                std::sync::TryLockError::WouldBlock => {
                    BridgeAuthoredBranchDenial::RegistryContactHeld { branch: target }
                }
                std::sync::TryLockError::Poisoned(_) => {
                    BridgeAuthoredBranchDenial::Poisoned { branch: target }
                }
            })
        })?;
        match slots.entry(target) {
            Entry::Occupied(_) => return Err(D::TargetOccupied { branch: target }),
            Entry::Vacant(entry) => {
                entry.insert(BranchSlot::Reserved {
                    memory: entry_memory,
                });
            }
        }
        drop(slots);
        let inherited = serving.lowerings.values().cloned().collect();
        serving.posture = BranchPosture::ForkPreparing { target };
        drop(serving);
        Ok(Box::new(Pending {
            reservation: Reservation {
                registry: Arc::clone(&self.branches),
                source: projection,
                target,
                phase: Phase::Preparing,
            },
            program: Arc::clone(&self.program),
            inherited,
            prepared: None,
            _memory: memory,
            refusal: Arc::new(Mutex::new(None)),
        }))
    }
}
fn fork_result(
    result: Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial>,
    refusal: Arc<Mutex<Option<D>>>,
) -> Result<SignalDurableAdvanceTask, D> {
    result.map_err(|cause| {
        refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .unwrap_or(D::Signal(Box::new(cause)))
    })
}
fn size_denial(_: crate::conditional_execution::retention::BridgeRetentionDenial) -> D {
    D::Readmission(Box::new(BridgeAuthoredReadmissionDenial::section(
        super::BridgeAuthoredSectionDenial::SizeLimit,
    )))
}
fn request_denial(cause: worth_execution::WorkCeilingDenial) -> D {
    D::Readmission(Box::new(BridgeAuthoredReadmissionDenial::section(
        super::BridgeAuthoredSectionDenial::Request(cause),
    )))
}
impl Pending {
    fn prepare_projection(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalForkPublicationContext<'_>,
    ) -> Result<worth_signal::facade::durability::SignalBridgeAuthoredSection, D> {
        {
            let source = self
                .reservation
                .source
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if source.section.as_signal_section().as_bytes() != context.predecessor().bridge_bytes()
            {
                return Err(D::SourceChanged);
            }
        }
        let section = BridgeAuthoredSection::capture_fork(execution, &context)
            .map_err(|cause| D::Readmission(Box::new(cause.into())))?;
        let _scratch = execution
            .reserve_retained_memory(
                crate::conditional_execution::retention::array_charge::<
                    worth_signal::facade::durability::SignalReadmittedConditionalContract,
                >(self.inherited.len())
                .map_err(size_denial)?,
            )
            .map_err(request_denial)?;
        let mut contracts = self
            .inherited
            .iter()
            .map(|lowering| context.readmit_inherited_contract(lowering.signal_contract()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|cause| D::Signal(Box::new(cause)))?;
        contracts.sort_by_key(|contract| contract.occurrence().node());
        let view = context
            .installation(&contracts, section.as_signal_section().as_bytes())
            .map_err(|cause| D::Signal(Box::new(cause)))?;
        let mut bindings = bindings::InheritedBindings {
            source: Arc::clone(&self.reservation.source),
            inherited: &self.inherited,
        };
        let mut installation = Installation {
            program: Arc::clone(&self.program),
            bindings: &mut bindings,
            branches: BTreeMap::new(),
            refusal: None,
        };
        let prepared = installation
            .prepare(view, execution)
            .map_err(|cause| D::Readmission(Box::new(cause)))?;
        let projection =
            crate::conditional_execution::runtime_builder::authored_readmission::finish(
                prepared,
                context.observation().branch_id().clone(),
                execution,
            )
            .map_err(|cause| D::Readmission(Box::new(cause)))?;
        self.reservation.phase = Phase::Prepared(context.original());
        self.reservation
            .source
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .posture = BranchPosture::Pending {
            original: context.original(),
        };
        self.prepared = Some(projection);
        Ok(section.into_signal_section())
    }
}
impl SignalDurableForkPublication for Pending {
    fn prepare(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalForkPublicationContext<'_>,
    ) -> Result<
        worth_signal::facade::durability::SignalBridgeAuthoredSection,
        SignalDurableAdvanceDenial,
    > {
        self.prepare_projection(execution, context)
            .map_err(|cause| {
                *self
                    .refusal
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cause);
                SignalDurableAdvanceDenial::DefinitionSectionRefused
            })
    }
}
impl SignalDurableProjectionPublication for Pending {
    fn committed(mut self: Box<Self>, branch: &SignalDurableBranchHandle) {
        let original = match self.reservation.phase {
            Phase::Prepared(original) => original,
            Phase::Preparing | Phase::Committed(_) | Phase::Installed => {
                unreachable!("ACK follows actual preparation")
            }
        };
        self.reservation.phase = Phase::Committed(original);
        assert_eq!(branch.descriptor().branch(), self.reservation.target);
        let projection = self
            .prepared
            .take()
            .expect("exact target prepared before I/O");
        let mut slots = self
            .reservation
            .registry
            .slots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = slots
            .get_mut(&self.reservation.target)
            .expect("reserved target survives original");
        let memory = match slot {
            BranchSlot::Reserved { memory } => memory.take(),
            BranchSlot::Serving { .. } | BranchSlot::ReadmissionRequired(_) => {
                unreachable!("original owns reservation")
            }
        };
        *slot = BranchSlot::Serving {
            projection,
            _memory: memory,
        };
        drop(slots);
        self.reservation
            .source
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .posture = BranchPosture::Serving;
        self.reservation.phase = Phase::Installed;
    }
}
