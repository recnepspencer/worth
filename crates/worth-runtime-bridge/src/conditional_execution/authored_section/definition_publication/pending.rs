//! The original Signal task owns both the prepared projection and its reservation.
use super::*;
use crate::conditional_execution::authored_section::{
    definition::BridgeReadmittedDefinitionSource, readmission::Program, BridgeAuthoredSection,
};
use crate::conditional_execution::{
    provider_admission::BridgeConditionalProviderAdmission,
    semantic_observation_plan::BridgeConditionalSemanticObservationPlan, BridgeConditionalLocation,
    BridgeConditionalProviderSet, BridgeOwnedSignalRuntime,
};
use worth_signal::facade::durability::{
    SignalDefinitionPublicationContext, SignalDurableDefinitionPublication,
};

pub(super) struct Pending {
    reservation: Reservation,
    program: Arc<Program>,
    predecessor: Arc<BridgeInstalledConditionalLowering>,
    location: Option<BridgeConditionalLocation>,
    providers: Option<BridgeConditionalProviderSet>,
    admission: Option<BridgeConditionalProviderAdmission>,
    plan: Option<BridgeConditionalSemanticObservationPlan>,
    retention: Arc<conditional::retention::BridgeRetentionLedger>,
    backing: Arc<Backing>,
    _preparation: Option<worth_execution::ExecutionMemoryReservation>,
    refusal: Arc<Mutex<Option<D>>>,
    prepared: Option<(
        BridgeAuthoredSection,
        Arc<BridgeInstalledConditionalLowering>,
    )>,
}
impl Pending {
    pub(super) fn new(
        reservation: Reservation,
        program: Arc<Program>,
        predecessor: Arc<BridgeInstalledConditionalLowering>,
        location: BridgeConditionalLocation,
        providers: BridgeConditionalProviderSet,
        admission: BridgeConditionalProviderAdmission,
        plan: Option<BridgeConditionalSemanticObservationPlan>,
        retention: Arc<conditional::retention::BridgeRetentionLedger>,
        backing: Arc<Backing>,
        preparation: Option<worth_execution::ExecutionMemoryReservation>,
        refusal: Arc<Mutex<Option<D>>>,
    ) -> Self {
        Self {
            reservation,
            program,
            predecessor,
            location: Some(location),
            providers: Some(providers),
            admission: Some(admission),
            plan,
            retention,
            backing,
            _preparation: preparation,
            refusal,
            prepared: None,
        }
    }
    pub(super) fn claimant(&self) -> worth_signal::facade::SignalAspectLoweringOwner {
        self.program.bridge.signal_aspect_lowering_owner.clone()
    }
    fn prepare_source(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalDefinitionPublicationContext<'_>,
    ) -> Result<worth_signal::facade::durability::SignalBridgeAuthoredSection, D> {
        let branch_identity = context.occurrence().node().branch();
        let admission = self
            .admission
            .take()
            .expect("one original prepares its projection once");
        let location = self
            .location
            .take()
            .expect("one original owns its location");
        let providers = self
            .providers
            .take()
            .expect("one original owns its provider set");
        let section = BridgeAuthoredSection::capture_definition_successor(
            execution,
            context.predecessor(),
            context.occurrence(),
            admission.contract(),
            &location,
        )
        .map_err(|cause| D::Section(Box::new(cause)))?;
        let key = conditional::lowering_registry::BridgeConditionalLoweringKey::new(
            self.predecessor.registry_key.branch().clone(),
            context.contract(),
        );
        let projection = conditional::lowering_identity::installed_lowering_identity(
            self.program.bridge.signal_runtime_key,
            key.branch(),
            context.contract(),
            admission.contract().identity(),
            &self.predecessor.correspondences,
        );
        let bytes = conditional::retention::definition_candidate_retained_bytes(
            key.branch(),
            admission.contract(),
            &location,
            &providers,
            projection.len(),
            self.predecessor.correspondence_count(),
            self.plan.as_ref(),
        )
        .map_err(|cause| {
            D::Conditional(Box::new(
                conditional::observation_retention::retention_denial(cause),
            ))
        })?;
        let source_bytes = conditional::retention::arc_charge::<BridgeReadmittedDefinitionSource>()
            .map_err(|_| D::Section(Box::new(BridgeAuthoredSectionDenial::SizeLimit)))?;
        self.backing
            .grow(
                execution,
                bytes
                    .checked_add(source_bytes)
                    .ok_or(D::Section(Box::new(BridgeAuthoredSectionDenial::SizeLimit)))?,
            )
            .map_err(|cause| D::Readmission(Box::new(cause)))?;
        let charge = Arc::new(self.retention.reserve_definition_candidate(bytes).map_err(
            |cause| {
                D::Conditional(Box::new(
                    conditional::observation_retention::retention_denial(cause),
                ))
            },
        )?);
        let location = Arc::new(location);
        let source = BridgeReadmittedDefinitionSource::published(
            Arc::clone(admission.shared_contract()),
            Arc::clone(&location),
            Arc::clone(&self.backing),
        );
        let lowering = BridgeOwnedSignalRuntime::build_installed_lowering(
            &self.program.bridge,
            key.clone(),
            location,
            Arc::clone(&self.predecessor.correspondences),
            self.plan.take(),
            context.contract().clone(),
            Some(context.custody().clone()),
            providers,
            admission,
            Some(charge),
            Some(source),
            Default::default(),
        );
        let mut branch = self
            .reservation
            .projection
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !branch.lowerings.reserve_owned_slot(key.clone()) {
            return Err(D::ProjectionSlotOccupied {
                branch: branch_identity,
                node: context.contract().node(),
                generation: context.contract().generation(),
            });
        }
        self.reservation.slot = Some(key);
        self.reservation.phase = ReservationPhase::Prepared {
            original: context.original(),
        };
        branch.posture = BranchPosture::Pending {
            original: context.original(),
        };
        let bytes = section.as_signal_section().clone();
        self.prepared = Some((section, lowering));
        Ok(bytes)
    }
}
impl SignalDurableDefinitionPublication for Pending {
    fn prepare(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        context: SignalDefinitionPublicationContext<'_>,
    ) -> Result<
        worth_signal::facade::durability::SignalBridgeAuthoredSection,
        SignalDurableAdvanceDenial,
    > {
        match self.prepare_source(execution, context) {
            Ok(section) => Ok(section),
            Err(cause) => {
                *self
                    .refusal
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cause);
                Err(SignalDurableAdvanceDenial::DefinitionSectionRefused)
            }
        }
    }
}

impl worth_signal::facade::durability::SignalDurableProjectionPublication for Pending {
    fn committed(mut self: Box<Self>, branch: &SignalDurableBranchHandle) {
        let original = match self.reservation.phase {
            ReservationPhase::Prepared { original } => original,
            ReservationPhase::Preparing
            | ReservationPhase::Committed { .. }
            | ReservationPhase::Installed => {
                unreachable!("only a prepared original receives committed cutover")
            }
        };
        self.reservation.phase = ReservationPhase::Committed { original };
        let (section, lowering) = self
            .prepared
            .take()
            .expect("prepared before the first journal effect");
        assert_eq!(section.branch(), branch.descriptor().branch());
        let mut projection = self
            .reservation
            .projection
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old = projection.lowerings.remove(&self.predecessor.registry_key);
        self.predecessor.lease.revoke_liveness();
        projection
            .lowerings
            .install(lowering.registry_key.clone(), lowering);
        projection.section = section;
        projection.posture = BranchPosture::Serving;
        self.reservation.phase = ReservationPhase::Installed;
        drop(projection);
        drop(old);
    }
}
