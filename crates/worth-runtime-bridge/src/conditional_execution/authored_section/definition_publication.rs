//! Stage one exact successor beside the original Signal task, without an effect.
use super::{
    branch_projection::{BranchPosture, BranchProjection},
    readmission::backing::Backing,
    BridgeAuthoredBranchDenial, BridgeAuthoredReadmissionDenial, BridgeAuthoredSectionDenial,
    BridgeReadmittedAuthoredAssembly,
};
use crate::conditional_execution::{
    self as conditional, BridgeConditionalDenial, BridgeInstalledConditionalLowering,
    BridgeOwnedConditionalInstallationRequest,
};
use std::sync::{Arc, Mutex};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableSignalBranchIdentity;
use worth_signal::facade::durability::{
    SignalDurableAdvanceDenial, SignalDurableAdvanceTask, SignalDurableBranchHandle,
};
mod pending;

#[derive(Debug)]
pub enum BridgeDurableDefinitionPublicationDenial {
    Branch(BridgeAuthoredBranchDenial),
    NotCurrent {
        branch: DurableSignalBranchIdentity,
        node: worth_signal::facade::NodeId,
    },
    IdentityOrCorrespondenceChanged,
    ProjectionSlotOccupied {
        branch: DurableSignalBranchIdentity,
        node: worth_signal::facade::NodeId,
        generation: u64,
    },
    Conditional(Box<BridgeConditionalDenial>),
    Readmission(Box<BridgeAuthoredReadmissionDenial>),
    Section(Box<BridgeAuthoredSectionDenial>),
    Signal(Box<SignalDurableAdvanceDenial>),
}
type D = BridgeDurableDefinitionPublicationDenial;
enum ReservationPhase {
    Preparing,
    Prepared {
        original: worth_foundational::facade::SignalBranchAdvanceIdentity,
    },
    Committed {
        original: worth_foundational::facade::SignalBranchAdvanceIdentity,
    },
    Installed,
}
struct Reservation {
    projection: Arc<BranchProjection>,
    phase: ReservationPhase,
    predecessor_lease: Arc<conditional::liveness::BridgeConditionalLoweringLease>,
    slot: Option<conditional::lowering_registry::BridgeConditionalLoweringKey>,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        let readmission = match self.phase {
            ReservationPhase::Installed => return,
            ReservationPhase::Preparing | ReservationPhase::Prepared { .. } => None,
            ReservationPhase::Committed { original } => Some(original),
        };
        let mut branch = self
            .projection
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(slot) = self.slot.take() {
            if let Some(lowering) = branch.lowerings.remove(&slot) {
                lowering.lease.revoke_liveness();
            }
        }
        branch.posture = match readmission {
            Some(original) => {
                self.predecessor_lease.revoke_liveness();
                BranchPosture::ReadmissionRequired { original }
            }
            None => BranchPosture::Serving,
        };
    }
}
mod preparation;
use preparation::PreparedDefinition;

impl BridgeReadmittedAuthoredAssembly {
    pub fn begin_conditional_definition_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Result<SignalDurableAdvanceTask, D> {
        let PreparedDefinition {
            pending,
            definition,
            refusal,
        } = self.prepare_definition_publication(
            execution,
            expected.descriptor().branch(),
            predecessor,
            request,
        )?;
        let result = self.signal.mutation_port().begin_definition_publication(
            *execution,
            expected,
            &mut (),
            pending.claimant(),
            predecessor.signal_contract().clone(),
            definition,
            pending,
        );
        definition_result(result, refusal)
    }
    pub fn begin_composite_conditional_definition_publication<F>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        binding: worth_signal::facade::durability::SignalCompositeAdvanceBinding,
        grant: worth_proof::AdmittedCompositeSignalPublication,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
        apply: F,
    ) -> Result<SignalDurableAdvanceTask, D>
    where
        F: FnOnce(
            &mut worth_signal::facade::SignalTransaction<'_, (), (), (), (), ()>,
        ) -> Result<(), worth_signal::facade::SignalError>,
    {
        let PreparedDefinition {
            pending,
            definition,
            refusal,
        } = self.prepare_definition_publication(
            execution,
            binding.source().descriptor().branch(),
            predecessor,
            request,
        )?;
        let result = self
            .signal
            .mutation_port()
            .begin_composite_definition_publication(
                *execution,
                binding,
                grant,
                &mut (),
                pending.claimant(),
                predecessor.signal_contract().clone(),
                definition,
                pending,
                apply,
            );
        definition_result(result, refusal)
    }
}
fn definition_result(
    result: Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial>,
    refusal: Arc<Mutex<Option<D>>>,
) -> Result<SignalDurableAdvanceTask, D> {
    match result {
        Ok(task) => Ok(task),
        Err(cause) => Err(refusal
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .unwrap_or(D::Signal(Box::new(cause)))),
    }
}
