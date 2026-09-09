use super::WorthUiActiveApplicationSession;
use crate::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiPresentationDeadline,
    UiSurfaceBindingCoordinatePosture, UiSurfaceBindingGeneration, UiSurfaceBindingProfile,
};
use std::collections::HashMap;

use super::native_observation_settlement::UiNativeObservationIngressSettlement;

#[cfg(any(test, feature = "certification-support"))]
#[path = "native_application_shell/certification.rs"]
mod certification;
#[path = "native_application_shell/component_presence.rs"]
mod component_presence;
pub(crate) use component_presence::UiNativeComponentPresenceProgress;
#[path = "native_application_shell/launch.rs"]
mod launch;
#[path = "native_application_shell/motion_sampling.rs"]
mod motion_sampling;
#[path = "native_application_shell/observation_clock.rs"]
mod observation_clock;
#[path = "native_application_shell/occurrence_geometry.rs"]
mod occurrence_geometry;
#[path = "native_application_shell/presentation_attribution.rs"]
mod presentation_attribution;
#[path = "native_application_shell/presentation_recovery.rs"]
mod presentation_recovery;
pub use presentation_recovery::{
    WorthUiNativePhysicalPresentationRecovery, WorthUiNativePresentationRecoveryDenial,
};
#[path = "native_application_shell/query_close.rs"]
mod query_close;
pub(crate) use query_close::UiNativeApplicationQueryCloseObservation;
#[path = "native_application_shell/service_inspection.rs"]
mod service_inspection;
pub use service_inspection::WorthUiNativeReducedMotionPosture;
#[path = "native_application_shell/shutdown.rs"]
mod shutdown;
#[path = "native_application_shell/viewport_measurement.rs"]
mod viewport_measurement;
pub use occurrence_geometry::{
    UiNativeMountedComponentLayoutInput, UiNativeMountedRegionLayoutInput,
};
pub use shutdown::{WorthUiNativeApplicationCleanup, WorthUiNativeApplicationShutdownReceipt};

/// High-level native lifecycle for one downstream application composition root.
pub struct WorthUiNativeApplicationShell {
    pub(super) session: Box<WorthUiActiveApplicationSession>,
    binding: UiSurfaceBindingGeneration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    scale_factor_milli: u32,
    mounted_rows: Vec<NativeMountedRow>,
    mounted_row_indices: HashMap<Box<str>, usize>,
    observed_viewport_basis: Option<viewport_measurement::UiNativeViewportBasis>,
    pending_viewport_basis: Option<viewport_measurement::UiNativeViewportBasis>,
    pending_surface_reconciliation: Option<crate::mounting::UiMountedSurfaceReconciliationBinding>,
    runtime_derived_state_reconstruction:
        Option<worth_ui_host_native::UiNativeClientDerivedStateReconstructionObservation>,
    pub(super) pending_managed_rebind:
        Option<super::native_managed_rebind::WorthUiNativePendingManagedRebind>,
    pub(super) retained_portal_dismissal:
        Option<super::native_managed_rebind::UiRetainedPortalDismissalRequest>,
    pending_component_presence: Option<component_presence::UiNativePendingComponentPresence>,
    pub(super) managed_rebind_completion_tick: u64,
    reduced_motion_posture: WorthUiNativeReducedMotionPosture,
}

struct NativeMountedRow {
    authored_semantic_identity: Box<str>,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    latest_mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
}

#[derive(Debug)]
pub enum WorthUiNativeApplicationShellLaunchDenial {
    RuntimeLaunch,
    RuntimeLaunchCleanup(Box<crate::runtime::WorthUiRuntimeLaunchDenial>),
    SemanticSurfaceCreation,
    HostSurfaceRegistration,
    MountedInstanceCreation,
    ViewportAllocation(Box<super::WorthUiMountedAllocationEstablishmentDenial>),
    OccurrenceGeometry(crate::mounting::UiMountedOccurrenceGeometryDenial),
    ApplicationCleanup(Box<WorthUiNativeApplicationCleanup>),
}

impl WorthUiNativeApplicationShell {
    pub(crate) fn admit_native_observation_batches(
        &mut self,
        reachability: worth_ui_host_native::UiNativeInputReachability,
    ) -> UiNativeObservationIngressSettlement {
        let pending_portal_transition = self
            .pending_managed_rebind
            .as_ref()
            .is_some_and(|pending| pending.carries_portal_intent_consequence());
        self.session
            .drain_and_admit_host_observation_batches(reachability, pending_portal_transition)
    }

    pub(crate) fn cancel_mounted_presentation(
        &mut self,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    ) -> crate::mounting::UiMountedFrameOutcome {
        self.session.cancel_mounted_presentation(in_flight)
    }

    pub(crate) fn rebind_native_surface_scale(
        &mut self,
        scale_factor_milli: u32,
    ) -> Result<(), ()> {
        if self.pending_surface_reconciliation.is_some() {
            return Err(());
        }
        let affected = self.binding;
        let scale_changed = self.scale_factor_milli != scale_factor_milli;
        let profile = UiSurfaceBindingProfile::new(
            scale_factor_milli,
            UiSurfaceBindingCoordinatePosture::LogicalPoints,
            1,
        )
        .map_err(|_| ())?;
        let rebind = |session: &mut WorthUiActiveApplicationSession| {
            session.rebind_host_surface_with_interaction_receipt(
                self.binding,
                UiHostSurfacePresentationMode::NativeDisplay,
                profile,
            )
        };
        let rebound = match rebind(&mut self.session) {
            Ok(rebound) => rebound.binding(),
            Err(super::UiSurfaceRebindInteractionDenial::BeforeMutation(
                crate::mounting::UiMountedIdentityDenial::HostSurfaceTruthIndeterminate,
            )) => {
                if self
                    .session
                    .recover_indeterminate_host_surface(self.surface)
                    .is_err()
                {
                    return Err(());
                }
                match rebind(&mut self.session) {
                    Ok(rebound) => rebound.binding(),
                    Err(_) => return Err(()),
                }
            }
            Err(_) => return Err(()),
        };
        self.binding = rebound.binding_generation();
        self.scale_factor_milli = scale_factor_milli;
        self.observe_native_viewport_binding_successor(scale_changed);
        self.pending_surface_reconciliation = Some(
            crate::mounting::UiMountedSurfaceReconciliationBinding::new(affected, self.binding),
        );
        Ok(())
    }

    pub fn update_intent_boolean_fact(
        &mut self,
        fact: &crate::facade::intent::UiIntentApplicationFact<
            crate::facade::intent::UiIntentBoolean,
        >,
        value: bool,
    ) -> Result<
        crate::facade::intent::UiIntentApplicationFactUpdateReceipt,
        crate::facade::intent::UiIntentApplicationFactUpdateDenial,
    > {
        self.session.update_intent_boolean_fact(fact, value)
    }

    pub fn update_intent_unsigned64_fact(
        &mut self,
        fact: &crate::facade::intent::UiIntentApplicationFact<
            crate::facade::intent::UiIntentUnsigned64,
        >,
        value: u64,
    ) -> Result<
        crate::facade::intent::UiIntentApplicationFactUpdateReceipt,
        crate::facade::intent::UiIntentApplicationFactUpdateDenial,
    > {
        self.session.update_intent_unsigned64_fact(fact, value)
    }

    pub const fn rebind_deadline_at(
        &self,
        tick: u64,
    ) -> crate::runtime::rebind::UiRebindSessionDeadline {
        self.session.rebind_deadline_at(tick)
    }

    pub const fn rebind_cancellation_request(
        &self,
    ) -> crate::runtime::rebind::UiRebindCancellationRequest {
        self.session.rebind_cancellation_request()
    }

    /// Execute and present one ordinary native frame.
    pub fn present_frame(
        &mut self,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<UiMountedFrameOutcome, super::WorthUiMountedFrameExecutionStop<'_>> {
        self.settle_pending_native_viewport_measurements()?;
        if let Some(replacement) = self.pending_surface_reconciliation {
            let replacements = [replacement];
            let outcome = self
                .session
                .execute_mounted_rebound_frame_with_application_presentation(
                    &replacements,
                    UiPresentationDeadline::at_tick(deadline_tick),
                    now_tick,
                )?;
            if surface_reconciliation_settled(&outcome) {
                self.pending_surface_reconciliation = None;
            }
            return Ok(outcome);
        }
        let outcome = self
            .session
            .execute_mounted_frame_with_application_presentation(
                UiPresentationDeadline::at_tick(deadline_tick),
                now_tick,
            )?;
        Ok(outcome)
    }

    pub(crate) fn prepare_frame(
        &mut self,
    ) -> Result<crate::mounting::UiPreparedMountedFrame, super::WorthUiMountedFrameExecutionStop<'_>>
    {
        self.settle_pending_native_viewport_measurements()?;
        let request = self.session.mounted_frame_request();
        if let Some(replacement) = self.pending_surface_reconciliation {
            let replacements = [replacement];
            let frame = self
                .session
                .prepare_mounted_reconciliation_frame_with_application_presentation(
                    request,
                    &replacements,
                    |_| {},
                )?;
            return Ok(frame);
        }
        self.session
            .prepare_mounted_frame_with_application_presentation(request, |_| {})
    }

    pub(crate) fn prepare_superseding_frame(
        &mut self,
        predecessor: &crate::mounting::UiPreparedMountedFrame,
    ) -> Result<crate::mounting::UiPreparedMountedFrame, super::WorthUiMountedFrameExecutionStop<'_>>
    {
        self.settle_pending_native_viewport_measurements()?;
        let request = self.session.mounted_frame_request();
        self.session
            .prepare_mounted_superseding_frame_with_application_presentation(
                request,
                predecessor,
                |_| {},
            )
    }

    pub(crate) fn present_prepared_frame(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        deadline_tick: u64,
        now_tick: u64,
    ) -> UiMountedFrameOutcome {
        let outcome = self.session.present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(deadline_tick),
            now_tick,
        );
        self.settle_surface_reconciliation(&outcome);
        outcome
    }

    pub(crate) fn present_prepared_superseding_frame(
        &mut self,
        frame: crate::mounting::UiPreparedMountedFrame,
        predecessor: crate::mounting::UiMountedSupersedingPresentationBasis,
        deadline_tick: u64,
        now_tick: u64,
    ) -> UiMountedFrameOutcome {
        let outcome = self
            .session
            .present_prepared_superseding_mounted_frame_internal(
                frame,
                predecessor,
                UiPresentationDeadline::at_tick(deadline_tick),
                now_tick,
            );
        self.settle_surface_reconciliation(&outcome);
        outcome
    }

    pub fn apply_component_semantic_text(
        &mut self,
        changes: &[super::UiNativeComponentSemanticTextChange],
    ) -> Result<(), super::UiNativeApplicationProgramDenial> {
        self.session
            .admit_application_semantic_text(changes)
            .map_err(|_| super::UiNativeApplicationProgramDenial::SemanticTextUpdateRejected)
    }

    pub(crate) fn apply_theme_token_values(
        &mut self,
        changes: &[super::UiNativeThemeTokenValueChange],
    ) -> Result<(), ()> {
        self.session.admit_application_theme_values(changes)
    }

    pub fn complete_frame_presentation(
        &mut self,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        now_tick: u64,
    ) -> UiMountedFrameOutcome {
        let outcome = self
            .session
            .complete_mounted_presentation(in_flight, now_tick);
        self.settle_surface_reconciliation(&outcome);
        outcome
    }

    pub(crate) fn admit_duplicate_native_presentation_observation(
        &mut self,
        presentation: worth_ui_host_native::UiNativePhysicalPresentationCorrelation,
    ) -> Result<(), ()> {
        self.session
            .admit_duplicate_native_presentation_observation(presentation)
    }

    pub(crate) fn retry_rejected_frame_presentation(
        &mut self,
        rejected: crate::mounting::UiMountedRejectedFrame,
        deadline: UiPresentationDeadline,
        now_tick: u64,
    ) -> UiMountedFrameOutcome {
        let outcome = self.session.present_prepared_mounted_frame_internal(
            rejected.into_frame(),
            deadline,
            now_tick,
        );
        self.settle_surface_reconciliation(&outcome);
        outcome
    }

    pub(super) fn settle_surface_reconciliation(&mut self, outcome: &UiMountedFrameOutcome) {
        if surface_reconciliation_settled(outcome) {
            self.pending_surface_reconciliation = None;
        }
    }

    pub fn generation_identity(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        self.session.generation_identity()
    }
}

fn surface_reconciliation_settled(outcome: &UiMountedFrameOutcome) -> bool {
    matches!(
        outcome,
        UiMountedFrameOutcome::Published(_)
            | UiMountedFrameOutcome::Unchanged(_)
            | UiMountedFrameOutcome::Reconciled(_)
    )
}
