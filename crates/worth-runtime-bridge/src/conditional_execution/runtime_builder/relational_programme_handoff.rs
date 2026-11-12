//! D16's Relational-only successor reuses its exact authored owner, not a companion graph.
use super::BridgeConditionalRuntimeBuilder;
use crate::conditional_execution::{
    authored_section::{branch_registry::BranchSlot, readmission::Program},
    retention, BridgeReadmittedAuthoredAssembly,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
#[derive(Debug)]
pub enum BridgeAuthoredProgrammeHandoffDenial {
    AuthoredSignalProgramme,
    FreshSignalProgramme,
    SourceProfileMismatch,
    PublicationAdvanced,
    BranchUnavailable,
    SignalSource(Box<worth_signal::facade::durability::SignalDurableSourceDenial>),
    SignalPublicationCatalog(Box<worth_signal::facade::durability::SignalOwnerReadmissionDenial>),
    SignalGate(Box<worth_signal::facade::durability::SignalAuthoredHomeStopDenial>),
    Request(worth_execution::WorkCeilingDenial),
    Retention,
    SizeLimit,
}
use BridgeAuthoredProgrammeHandoffDenial as D;
impl BridgeReadmittedAuthoredAssembly {
    /// Called before a D16 intent: nonempty Signal programme migration remains
    /// owned by its later migration phase. This checks the actual authored owner.
    pub fn preflight_relational_programme_transition(&self) -> Result<(), D> {
        if self
            .signal
            .source_port()
            .topology_observations()
            .map_err(|cause| D::SignalSource(Box::new(cause)))?
            .iter()
            .any(|(count, _)| *count != 0)
        {
            return Err(D::AuthoredSignalProgramme);
        }
        let slots = self
            .branches
            .slots
            .try_lock()
            .map_err(|_| D::BranchUnavailable)?;
        for slot in slots.values() {
            match slot {
                BranchSlot::Serving { projection, .. } => {
                    let state = projection
                        .state
                        .try_lock()
                        .map_err(|_| D::BranchUnavailable)?;
                    if !matches!(state.posture, crate::conditional_execution::authored_section::branch_projection::BranchPosture::Serving)
                        || !state.lowerings.read().unwrap_or_else(std::sync::PoisonError::into_inner).is_empty_for_home_stop()
                        || state._dependencies.admitted_count() != 0 {
                        return Err(D::AuthoredSignalProgramme);
                    }
                }
                BranchSlot::Reserved { .. } | BranchSlot::ReadmissionRequired(_) => {
                    return Err(D::BranchUnavailable)
                }
            }
        }
        Ok(())
    }
    /// Both actual owners return on refusal. Every check and allocation precedes
    /// the programme switch; the durable Signal root and all its coordinates stay original.
    pub fn rebind_relational_programme(
        mut self: Box<Self>,
        mut builder: Box<BridgeConditionalRuntimeBuilder>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<Self>, (Box<Self>, Box<BridgeConditionalRuntimeBuilder>, D)> {
        match self.rebind_relational_programme_inner(&mut builder, execution) {
            Ok(()) => Ok(self),
            Err(cause) => Err((self, builder, cause)),
        }
    }
    fn rebind_relational_programme_inner(
        &mut self,
        builder: &mut BridgeConditionalRuntimeBuilder,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), D> {
        let published = self
            .signal
            .published_advance_count()
            .map_err(|cause| D::SignalPublicationCatalog(Box::new(cause)))?;
        self.preflight_relational_programme_transition()?;
        if !builder.managed_clocks.is_empty()
            || builder.runtime.owned_async_declaration_count() != 0
            || builder
                .runtime
                .lock_signal_runtime()
                .graph()
                .active_node_count()
                != 0
        {
            return Err(D::FreshSignalProgramme);
        }
        let bridge: &crate::facade::RuntimeBridge = &builder.runtime.bridge;
        if bridge.authoritative_source_profile()
            != self.program.bridge.authoritative_source_profile()
            || bridge.signal_runtime_key != self.program.bridge.signal_runtime_key
        {
            return Err(D::SourceProfileMismatch);
        }
        let program_memory = execution
            .reserve_retained_memory(retention::arc_charge::<Program>().map_err(|_| D::SizeLimit)?)
            .map_err(D::Request)?;
        let slots = self
            .branches
            .slots
            .try_lock()
            .map_err(|_| D::BranchUnavailable)?;
        let _preparation = execution.reserve_retained_memory((slots.len() as u64).checked_mul(std::mem::size_of::<(Arc<crate::conditional_execution::authored_section::branch_projection::BranchProjection>, Arc<retention::BridgeRetentionLedger>, Option<worth_execution::ExecutionMemoryReservation>)>() as u64).ok_or(D::SizeLimit)?).map_err(D::Request)?;
        let mut prepared = Vec::with_capacity(slots.len());
        for slot in slots.values() {
            match slot {
                BranchSlot::Serving { projection, .. } => {
                    let retention = retention::BridgeRetentionLedger::new(
                        bridge.policy.conditional_retention(),
                    )
                    .map_err(|_| D::Retention)?;
                    let memory = execution
                        .reserve_retained_memory(
                            retention::arc_charge::<retention::BridgeRetentionLedger>()
                                .map_err(|_| D::SizeLimit)?,
                        )
                        .map_err(D::Request)?;
                    prepared.push((Arc::clone(projection), retention, memory));
                }
                BranchSlot::Reserved { .. } | BranchSlot::ReadmissionRequired(_) => {
                    return Err(D::BranchUnavailable)
                }
            }
        }
        drop(slots);
        let gate = self
            .signal
            .prepare_home_stop(execution)
            .map_err(|cause| D::SignalGate(Box::new(cause)))?;
        if self
            .signal
            .published_advance_count()
            .map_err(|cause| D::SignalPublicationCatalog(Box::new(cause)))?
            != published
        {
            return Err(D::PublicationAdvanced);
        }
        // The caller owns this assembly uniquely; the actual Signal gate excludes
        // pending original callbacks. No failable operation follows template transfer.
        builder.runtime.revoke_conditional_liveness();
        builder.runtime.retention.close();
        let mut template = builder
            .runtime
            .bridge
            .take()
            .expect("validated original programme template");
        template.diagnostics = template.diagnostics.for_authored_installation();
        template.semantic_dependency_registry = Default::default();
        template.correspondence_allocations = Default::default();
        self.program = Arc::new(Program {
            bridge: template,
            installation: builder.program_owner.binding(),
            _memory: program_memory,
        });
        for (projection, retention, memory) in prepared {
            let mut state = projection
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.retention.close();
            state.retention = retention;
            state.programme_memory = memory;
        }
        drop(gate); // Resume this SAME canonical owner; no Signal Closed claim is made.
        Ok(())
    }
}
