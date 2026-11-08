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
impl BridgeReadmittedAuthoredAssembly {
    pub fn begin_conditional_definition_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        predecessor: &Arc<BridgeInstalledConditionalLowering>,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Result<SignalDurableAdvanceTask, D> {
        let branch = expected.descriptor().branch();
        let projection = self.branches.get(&branch).cloned().ok_or(D::Branch(
            BridgeAuthoredBranchDenial::UnknownBranch { branch },
        ))?;
        let mut serving = projection.serving(branch).map_err(D::Branch)?;
        if !serving
            .lowerings
            .get(&predecessor.registry_key)
            .is_some_and(|installed| Arc::ptr_eq(installed, predecessor))
        {
            return Err(D::NotCurrent {
                branch,
                node: predecessor.signal_node(),
            });
        }
        conditional::owned_installation::validate_dependency_shape(&request)
            .map_err(|cause| D::Conditional(Box::new(cause)))?;
        if request.contract.identity() != predecessor.contract().identity()
            || request.location != *predecessor.location()
            || request.dependencies.len() != predecessor.correspondence_count()
            || !request
                .dependencies
                .iter()
                .zip(predecessor.semantic_dependencies())
                .all(|(a, b)| a == b)
        {
            return Err(D::IdentityOrCorrespondenceChanged);
        }
        let bytes = conditional::retention::definition_candidate_retained_bytes(
            predecessor.registry_key.branch(),
            &request.contract,
            &request.location,
            &request.providers,
            predecessor.projection().label().len(),
            predecessor.correspondence_count(),
            None,
        )
        .map_err(|cause| {
            D::Conditional(Box::new(
                conditional::observation_retention::retention_denial(cause),
            ))
        })?;
        // Preparation and retained source have separate, original-request grants.
        let preparation = execution
            .reserve_retained_memory(
                bytes
                    .checked_add(std::mem::size_of::<pending::Pending>() as u64)
                    .ok_or(D::Section(Box::new(BridgeAuthoredSectionDenial::SizeLimit)))?,
            )
            .map_err(|cause| D::Section(Box::new(BridgeAuthoredSectionDenial::Request(cause))))?;
        let program = Arc::clone(&serving._program);
        let retention = Arc::clone(&serving.retention);
        serving.posture = BranchPosture::Preparing {
            node: predecessor.signal_node(),
        };
        let reservation = Reservation {
            projection: Arc::clone(&projection),
            phase: ReservationPhase::Preparing,
            predecessor_lease: Arc::clone(&predecessor.lease),
            slot: None,
        };
        drop(serving);
        let backing = Backing::new(execution).map_err(|cause| D::Readmission(Box::new(cause)))?;
        backing
            .grow(execution, bytes)
            .map_err(|cause| D::Readmission(Box::new(cause)))?;
        let admission = conditional::provider_admission::admit_provider_set(
            Arc::new(request.contract),
            &request.providers,
        )
        .map_err(|cause| D::Conditional(Box::new(cause)))?;
        let plan = conditional::semantic_observation_plan::compile_semantic_observation_plan_from_dependencies(
            admission.contract(), request.dependencies.iter(),
        ).map_err(|cause| D::Conditional(Box::new(cause)))?;
        let definition = conditional::lowering::lower_signal_contract(
            admission.contract(),
            predecessor.signal_dependency_aspects(),
            predecessor
                .signal_condition_aspects(admission.contract().condition_dependency_ordinals()),
        )
        .map_err(|cause| D::Conditional(Box::new(cause)))?;
        let refusal = Arc::new(Mutex::new(None));
        let pending = Box::new(pending::Pending::new(
            reservation,
            program,
            Arc::clone(predecessor),
            request.location,
            request.providers,
            admission,
            plan,
            retention,
            backing,
            preparation,
            Arc::clone(&refusal),
        ));
        let result = self.signal.mutation_port().begin_definition_publication(
            *execution,
            expected,
            &mut (),
            pending.claimant(),
            predecessor.signal_contract().clone(),
            definition,
            pending,
        );
        match result {
            Ok(task) => Ok(task),
            Err(cause) => Err(refusal
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .unwrap_or(D::Signal(Box::new(cause)))),
        }
    }
}
