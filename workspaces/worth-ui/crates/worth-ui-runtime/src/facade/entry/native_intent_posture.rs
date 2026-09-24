use super::{
    intent_consequence_observation::prepare_intent_consequence_observation,
    WorthUiActiveApplicationSession, WorthUiNativeApplicationShell, WorthUiNativeIntentPosture,
};

mod execution;

pub(in crate::facade::entry) use execution::DetachedNativeIntentPosturePending;
use execution::{
    NativeIntentPostureInFlight, NativeIntentPostureIndeterminate, NativeIntentPostureRejected,
    PreparedNativeIntentPostureRebind,
};

pub enum WorthUiNativeIntentPosturePublicationOutcome<'session> {
    Stopped(WorthUiNativeIntentPosturePublicationStop),
    Published(crate::runtime::rebind::UiRebindReceipt),
    InFlight(WorthUiNativeIntentPosturePublicationCompletion<'session>),
    RejectedBeforeEffects(WorthUiNativeIntentPosturePublicationRetry<'session>),
    Indeterminate(WorthUiNativeIntentPosturePublicationRecovery<'session>),
    InternalDefect(crate::runtime::rebind::UiRebindInternalDefectOutcome),
}

#[must_use = "posture publication must be completed or explicitly disposed"]
pub struct WorthUiNativeIntentPosturePublicationCompletion<'session> {
    state: Option<Box<NativeIntentPostureInFlight<'session>>>,
}

#[must_use = "indeterminate posture publication requires reconciliation or shutdown"]
pub struct WorthUiNativeIntentPosturePublicationRecovery<'session> {
    state: Option<Box<NativeIntentPostureIndeterminate<'session>>>,
}

#[derive(Debug)]
pub struct WorthUiNativeIntentPosturePublicationStop {
    reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
    host_rejections: Box<[worth_ui_host_contract::UiHostSurfacePresentationDenial]>,
}

#[must_use = "rejected posture publication retains an exact prepared retry"]
pub struct WorthUiNativeIntentPosturePublicationRetry<'session> {
    state: Option<Box<NativeIntentPostureRejected<'session>>>,
}

#[derive(Debug)]
pub enum WorthUiNativeManagedIntentPosturePublicationDenial {
    ManagedRebindAlreadyInFlight,
    ManagedRebindSessionMismatch,
    PredecessorReconstruction,
}

pub enum WorthUiNativeManagedIntentPosturePublicationOutcome {
    Published(crate::runtime::rebind::UiRebindReceipt),
    Pending,
    Stopped(super::native_managed_rebind::WorthUiNativeManagedRebindStop),
}

struct NativeIntentPostureTransfer {
    observation: super::intent_consequence_observation::WorthUiPreparedConsequenceObservationCommit,
    posture: crate::mounting::UiIntentPostureCommit,
}

impl WorthUiNativeApplicationShell {
    /// Publish a runtime-minted posture through the canonical non-source 3.12
    /// observation, planning, presentation, and commit path.
    pub fn publish_native_intent_posture(
        &mut self,
        posture: WorthUiNativeIntentPosture,
        now_tick: u64,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'_> {
        if self.pending_managed_rebind.is_some() {
            return stopped(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::RebindAdmission(
                    crate::runtime::rebind::UiRebindReservationDenial::AdmissionClosed,
                ),
            );
        }
        self.session.publish_native_intent_posture(
            posture,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            crate::runtime::rebind::UiRebindExecutionRequest::new(now_tick),
        )
    }

    /// Begin posture publication through the shell-owned native physical-progress lane.
    pub fn begin_managed_native_intent_posture_publication(
        &mut self,
        posture: WorthUiNativeIntentPosture,
        now_tick: u64,
    ) -> Result<
        WorthUiNativeManagedIntentPosturePublicationOutcome,
        WorthUiNativeManagedIntentPosturePublicationDenial,
    > {
        if self.pending_managed_rebind.is_some() {
            return Err(
                WorthUiNativeManagedIntentPosturePublicationDenial::ManagedRebindAlreadyInFlight,
            );
        }
        let outcome = self.publish_native_intent_posture(posture, now_tick);
        match super::native_managed_rebind::normalize_managed_intent_posture(outcome) {
            super::native_managed_rebind::ManagedIntentPostureNormalization::Published(receipt) => {
                Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Published(receipt))
            }
            super::native_managed_rebind::ManagedIntentPostureNormalization::Pending(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(
                        WorthUiNativeManagedIntentPosturePublicationDenial::
                            ManagedRebindSessionMismatch,
                    );
                }
                self.pending_managed_rebind = Some(
                    super::native_managed_rebind::WorthUiNativePendingManagedRebind::IntentPosture(
                        pending,
                    ),
                );
                Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Pending)
            }
            super::native_managed_rebind::ManagedIntentPostureNormalization::
                ReconstructionRequired(retry) =>
            {
                self.begin_intent_posture_predecessor_reconstruction(retry)
            }
            super::native_managed_rebind::ManagedIntentPostureNormalization::Indeterminate { recovery, frame } => {
                self.pending_managed_rebind = Some(
                    super::native_managed_rebind::WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame },
                );
                Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Pending)
            }
            super::native_managed_rebind::ManagedIntentPostureNormalization::Stopped(stop) => {
                Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Stopped(stop))
            }
        }
    }
}

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn publish_native_intent_posture(
        &mut self,
        posture: WorthUiNativeIntentPosture,
        policy: crate::runtime::rebind::UiRebindExecutionPolicy,
        execution: crate::runtime::rebind::UiRebindExecutionRequest,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'_> {
        let Some(bound) = self
            .intent_postures
            .bind_publication_order(posture.observation, posture.commit)
        else {
            return stopped(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::IntentPostureIdentityExhausted,
            );
        };
        let batch = crate::runtime::observation::UiIntentConsequenceObservationBatch::new(
            Some(bound),
            None,
            None,
        );
        let observation = match prepare_intent_consequence_observation(self, batch) {
            Ok(observation) => observation,
            Err(stop) => return stopped(stop.reason),
        };
        let change = self
            .application
            .classify_intent_consequence(self.identity, observation.set);
        let scope = match crate::runtime::rebind::UiAffectedScopeResolver::resolve_recoverable(
            change,
            self.identity,
            self.application.prepared_authority(),
        ) {
            Ok(scope) => scope,
            Err(stop) => {
                let (denial, _) = stop.into_parts();
                return stopped(
                    crate::runtime::intent_execution::UiIntentConsequenceStopReason::AffectedScope(
                        Box::new(denial),
                    ),
                );
            }
        };
        let lifecycle =
            match crate::runtime::rebind::UiIdentityLifecycleResolver::resolve_recoverable(scope) {
                Ok(lifecycle) => lifecycle,
                Err(stop) => {
                    let (denial, _) = stop.into_parts();
                    return stopped(
                        crate::runtime::intent_execution::UiIntentConsequenceStopReason::IdentityLifecycle(
                            Box::new(denial),
                        ),
                    );
                }
            };
        let plan = match self.application.compile_non_source_rebind_recoverable(
            self.identity,
            lifecycle,
            policy,
        ) {
            Ok(plan) => plan,
            Err(stop) => {
                let (denial, _) = stop.into_parts();
                return stopped(
                    crate::runtime::intent_execution::UiIntentConsequenceStopReason::Planning(
                        Box::new(denial),
                    ),
                );
            }
        };
        let posture = observation
            .posture
            .expect("posture-only observation retains one posture commit");
        match self.prepare_native_intent_posture_rebind(
            plan,
            execution,
            observation.progress,
            posture,
        ) {
            Ok(prepared) => prepared.execute(),
            Err(reason) => stopped(reason),
        }
    }

    fn prepare_native_intent_posture_rebind(
        &mut self,
        plan: crate::runtime::rebind::UiRebindPlan,
        request: crate::runtime::rebind::UiRebindExecutionRequest,
        observation: super::intent_consequence_observation::WorthUiClosedConsequenceObservation,
        posture: crate::mounting::UiIntentPostureCommit,
    ) -> Result<
        PreparedNativeIntentPostureRebind<'_>,
        crate::runtime::intent_execution::UiIntentConsequenceStopReason,
    > {
        let now_tick = request.now_tick();
        if !plan.has_non_source_semantic_proof() {
            return Err(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(
                    Box::new(
                        crate::runtime::rebind::UiRebindPreparationDenial::InvalidSemanticProof,
                    ),
                ),
            );
        }
        let reservation = crate::runtime::rebind::admit_plan(
            &self.rebind,
            crate::runtime::rebind::UiRebindFinalAdmissionBasis::new(
                self.identity,
                self.capabilities().digest().as_u64(),
                self.generation_identity(),
            ),
            &plan,
            request,
        )
        .map_err(|denial| {
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(Box::new(
                denial,
            ))
        })?;
        let (portal_overlay_revision, portal_overlays) = self.portal.as_ref().map_or_else(
            || (0, Vec::new()),
            |owner| (owner.revision(), owner.current_mounted_projection_inputs()),
        );
        let (frame, observation) = self
            .prepare_observed_intent_consequence_frame(
                plan.content().clone(),
                portal_overlay_revision,
                portal_overlays,
                observation,
            )
            .map_err(|denial| {
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(
                    Box::new(denial),
                )
            })?;
        let transfer = NativeIntentPostureTransfer {
            observation,
            posture,
        };
        Ok(PreparedNativeIntentPostureRebind {
            session: self,
            plan,
            reservation,
            frame,
            transfer,
            now_tick,
        })
    }
}

impl WorthUiNativeIntentPosturePublicationStop {
    pub const fn reason(&self) -> &crate::runtime::intent_execution::UiIntentConsequenceStopReason {
        &self.reason
    }

    pub fn host_rejections(&self) -> &[worth_ui_host_contract::UiHostSurfacePresentationDenial] {
        &self.host_rejections
    }
}

impl WorthUiNativeIntentPosturePublicationStop {
    pub(super) fn host_rejected(
        rejections: Box<[worth_ui_host_contract::UiHostSurfacePresentationDenial]>,
    ) -> Self {
        Self {
            reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason::
                HostRejectedBeforeEffects {
                    rejection_count: rejections.len(),
                },
            host_rejections: rejections,
        }
    }
}

pub(super) fn stopped<'session>(
    reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
    WorthUiNativeIntentPosturePublicationOutcome::Stopped(
        WorthUiNativeIntentPosturePublicationStop {
            reason,
            host_rejections: Box::new([]),
        },
    )
}
