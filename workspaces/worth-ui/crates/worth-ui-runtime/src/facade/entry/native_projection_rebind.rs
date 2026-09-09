use super::WorthUiNativeApplicationShell;

#[derive(Debug)]
pub enum WorthUiNativeProjectionRebindDenial {
    ObservationTurn(crate::runtime::observation::UiObservationTurnDenial),
    ObservationAdmission(crate::runtime::observation::UiObservationAdmissionDenial),
    Classification(crate::runtime::observation::UiChangeClassificationDenial),
    UnexpectedClassification,
    Scope(crate::runtime::rebind::UiAffectedScopeDenial),
    Identity(crate::runtime::rebind::UiIdentityLifecycleDenial),
    Planning(crate::runtime::rebind::UiRebindPlanningDenial),
    Preparation(crate::runtime::rebind::UiRebindPreparationDenial),
    ManagedRebindAlreadyInFlight,
    ManagedRebindSessionMismatch,
}

pub enum WorthUiNativeManagedProjectionRebindOutcome {
    Published(crate::runtime::rebind::UiRebindReceipt),
    Pending,
    Stopped(super::native_managed_rebind::WorthUiNativeManagedRebindStop),
}

impl WorthUiNativeApplicationShell {
    pub fn begin_projection_rebind(
        &mut self,
        request: crate::runtime::rebind::UiProjectionRebindRequest,
    ) -> Result<crate::runtime::rebind::UiRebindOutcome<'_>, WorthUiNativeProjectionRebindDenial>
    {
        let (observation, policy, execution) = request.into_parts();
        let admitted = admit_projection(&mut self.session, observation)?;
        let admitted = match admitted {
            NativeProjectionAdmission::Admitted(admitted) => admitted,
            NativeProjectionAdmission::Duplicate(receipt) => {
                return Ok(crate::runtime::rebind::UiRebindOutcome::Duplicate(receipt));
            }
            NativeProjectionAdmission::Superseded(receipt) => {
                return Ok(
                    crate::runtime::rebind::UiRebindOutcome::SupersededBeforeEffects(receipt),
                );
            }
        };
        let classified = self
            .session
            .classify_observations(admitted)
            .map_err(WorthUiNativeProjectionRebindDenial::Classification)?;
        let plan = plan_projection_rebind(&mut self.session, classified, policy)?;
        execute_projection_rebind(&mut self.session, plan, execution)
    }

    pub fn begin_managed_projection_rebind(
        &mut self,
        request: crate::runtime::rebind::UiProjectionRebindRequest,
    ) -> Result<WorthUiNativeManagedProjectionRebindOutcome, WorthUiNativeProjectionRebindDenial>
    {
        if self.pending_managed_rebind.is_some() {
            return Err(WorthUiNativeProjectionRebindDenial::ManagedRebindAlreadyInFlight);
        }
        let outcome = self.begin_projection_rebind(request)?;
        match super::native_managed_rebind::normalize_managed_outcome(outcome) {
            super::native_managed_rebind::ManagedRebindNormalization::Published(receipt) => Ok(
                WorthUiNativeManagedProjectionRebindOutcome::Published(receipt),
            ),
            super::native_managed_rebind::ManagedRebindNormalization::Pending(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeProjectionRebindDenial::ManagedRebindSessionMismatch);
                }
                self.pending_managed_rebind = Some(
                    super::native_managed_rebind::WorthUiNativePendingManagedRebind::Completion(
                        pending,
                    ),
                );
                Ok(WorthUiNativeManagedProjectionRebindOutcome::Pending)
            }
            super::native_managed_rebind::ManagedRebindNormalization::Stopped(stop) => {
                Ok(WorthUiNativeManagedProjectionRebindOutcome::Stopped(stop))
            }
        }
    }
}

enum NativeProjectionAdmission {
    Admitted(crate::runtime::observation::UiAdmittedObservationSet),
    Duplicate(crate::runtime::rebind::UiDuplicateObservationReceipt),
    Superseded(crate::runtime::rebind::UiRebindSupersededReceipt),
}

fn admit_projection(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    observation: worth_ui_query_binding::UiProjectionObservation,
) -> Result<NativeProjectionAdmission, WorthUiNativeProjectionRebindDenial> {
    let mut turn = session
        .begin_observation_turn()
        .map_err(WorthUiNativeProjectionRebindDenial::ObservationTurn)?;
    let identity = turn.identity();
    match turn.admit_projection_query(observation) {
        Ok(_) => {}
        Err(crate::runtime::observation::UiObservationAdmissionDenial::DuplicateOwnerOrder) => {
            return Ok(NativeProjectionAdmission::Duplicate(
                crate::runtime::rebind::UiDuplicateObservationReceipt::new(identity),
            ));
        }
        Err(crate::runtime::observation::UiObservationAdmissionDenial::HistoricalOwnerOrder) => {
            return Ok(NativeProjectionAdmission::Superseded(
                crate::runtime::rebind::UiRebindSupersededReceipt::before_effects(
                    crate::runtime::rebind::UiRebindStoppedPhase::ObservationAdmission,
                ),
            ));
        }
        Err(denial) => {
            return Err(WorthUiNativeProjectionRebindDenial::ObservationAdmission(
                denial,
            ));
        }
    }
    turn.seal()
        .map(NativeProjectionAdmission::Admitted)
        .map_err(WorthUiNativeProjectionRebindDenial::ObservationAdmission)
}

fn plan_projection_rebind(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    classified: crate::runtime::observation::UiChangeClassificationOutcome,
    policy: crate::runtime::rebind::UiRebindExecutionPolicy,
) -> Result<crate::runtime::rebind::UiRebindPlan, WorthUiNativeProjectionRebindDenial> {
    let change = match classified {
        crate::runtime::observation::UiChangeClassificationOutcome::Changed(change) => change,
        crate::runtime::observation::UiChangeClassificationOutcome::ObservedNoChange(_)
        | crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(_) => {
            return Err(WorthUiNativeProjectionRebindDenial::UnexpectedClassification);
        }
    };
    let lifecycle = session
        .resolve_affected_scope(change)
        .map_err(WorthUiNativeProjectionRebindDenial::Scope)?
        .resolve_identity_lifecycle()
        .map_err(WorthUiNativeProjectionRebindDenial::Identity)?;
    session
        .compile_rebind_plan(lifecycle, policy)
        .map_err(WorthUiNativeProjectionRebindDenial::Planning)
}

fn execute_projection_rebind<'session>(
    session: &'session mut crate::facade::WorthUiActiveApplicationSession,
    plan: crate::runtime::rebind::UiRebindPlan,
    execution: crate::runtime::rebind::UiRebindExecutionRequest,
) -> Result<crate::runtime::rebind::UiRebindOutcome<'session>, WorthUiNativeProjectionRebindDenial>
{
    let now_tick = execution.now_tick();
    let prepared = session
        .prepare_rebind(plan, execution)
        .map_err(WorthUiNativeProjectionRebindDenial::Preparation)?;
    Ok(prepared.execute(now_tick))
}
