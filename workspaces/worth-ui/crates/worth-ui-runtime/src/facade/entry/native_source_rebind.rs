use super::WorthUiNativeApplicationShell;

#[derive(Debug)]
pub enum WorthUiNativeSourceRebindDenial {
    Source(crate::runtime::UiSourceRebindAttemptFailure),
    ObservationTurn(crate::runtime::observation::UiObservationTurnDenial),
    ObservationAdmission(crate::runtime::observation::UiObservationAdmissionDenial),
    Classification(crate::runtime::observation::UiChangeClassificationDenial),
    Scope(crate::runtime::rebind::UiAffectedScopeDenial),
    Identity(crate::runtime::rebind::UiIdentityLifecycleDenial),
    Planning(crate::runtime::rebind::UiRebindPlanningDenial),
    Preparation(crate::runtime::rebind::UiRebindPreparationDenial),
    ManagedRebindAlreadyInFlight,
    ManagedRebindSessionMismatch,
}

pub enum WorthUiNativeManagedSourceRebindOutcome {
    Published(crate::runtime::rebind::UiRebindReceipt),
    Pending,
    Stopped(super::native_managed_rebind::WorthUiNativeManagedRebindStop),
}

impl WorthUiNativeSourceRebindDenial {
    pub const fn source_failure(&self) -> Option<&crate::runtime::UiSourceRebindAttemptFailure> {
        match self {
            Self::Source(failure) => Some(failure),
            _ => None,
        }
    }
}

impl WorthUiNativeApplicationShell {
    pub fn begin_managed_source_rebind(
        &mut self,
        request: crate::runtime::rebind::UiSourceRebindRequest,
    ) -> Result<WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial> {
        if self.pending_managed_rebind.is_some() {
            return Err(WorthUiNativeSourceRebindDenial::ManagedRebindAlreadyInFlight);
        }
        let outcome = self.begin_source_rebind(request)?;
        match super::native_managed_rebind::normalize_managed_outcome(outcome) {
            super::native_managed_rebind::ManagedRebindNormalization::Published(receipt) => {
                Ok(WorthUiNativeManagedSourceRebindOutcome::Published(receipt))
            }
            super::native_managed_rebind::ManagedRebindNormalization::Pending(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeSourceRebindDenial::ManagedRebindSessionMismatch);
                }
                self.pending_managed_rebind = Some(
                    super::native_managed_rebind::WorthUiNativePendingManagedRebind::Completion(
                        pending,
                    ),
                );
                Ok(WorthUiNativeManagedSourceRebindOutcome::Pending)
            }
            super::native_managed_rebind::ManagedRebindNormalization::Stopped(stop) => {
                Ok(WorthUiNativeManagedSourceRebindOutcome::Stopped(stop))
            }
        }
    }

    pub fn begin_source_rebind(
        &mut self,
        request: crate::runtime::rebind::UiSourceRebindRequest,
    ) -> Result<crate::runtime::rebind::UiRebindOutcome<'_>, WorthUiNativeSourceRebindDenial> {
        let (snapshot, policy, execution) = request.into_parts();
        let candidate = compile_source_candidate(&self.session, snapshot)?;
        let admitted = admit_source_candidate(&mut self.session, candidate)?;
        let admitted = match admitted {
            NativeSourceAdmission::Admitted(admitted) => admitted,
            NativeSourceAdmission::Duplicate(receipt) => {
                return Ok(crate::runtime::rebind::UiRebindOutcome::Duplicate(receipt));
            }
            NativeSourceAdmission::Superseded(receipt) => {
                return Ok(
                    crate::runtime::rebind::UiRebindOutcome::SupersededBeforeEffects(receipt),
                );
            }
        };
        let classified = classify_source_change(&mut self.session, admitted)?;
        let planned = plan_source_rebind(&mut self.session, classified, policy)?;
        match planned {
            NativeSourceRebindPlan::ObservedNoChange(receipt) => Ok(
                crate::runtime::rebind::UiRebindOutcome::ObservedNoChange(receipt),
            ),
            NativeSourceRebindPlan::Planned(plan) => {
                execute_source_rebind(&mut self.session, plan, execution)
            }
        }
    }
}

enum NativeSourceAdmission {
    Admitted(crate::runtime::observation::UiAdmittedObservationSet),
    Duplicate(crate::runtime::rebind::UiDuplicateObservationReceipt),
    Superseded(crate::runtime::rebind::UiRebindSupersededReceipt),
}

enum NativeSourceRebindPlan {
    ObservedNoChange(crate::runtime::observation::UiObservedNoChangeReceipt),
    Planned(crate::runtime::rebind::UiRebindPlan),
}

#[inline(never)]
fn compile_source_candidate(
    session: &crate::facade::WorthUiActiveApplicationSession,
    snapshot: crate::runtime::WorthUiSettledSourceSnapshot,
) -> Result<crate::runtime::WorthUiWatchedCandidateSubmission, WorthUiNativeSourceRebindDenial> {
    snapshot
        .attempt_source_rebind(session.capabilities())
        .into_candidate_submission()
        .map_err(WorthUiNativeSourceRebindDenial::Source)
}

#[inline(never)]
fn admit_source_candidate(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    candidate: crate::runtime::WorthUiWatchedCandidateSubmission,
) -> Result<NativeSourceAdmission, WorthUiNativeSourceRebindDenial> {
    let mut turn = session
        .begin_observation_turn()
        .map_err(WorthUiNativeSourceRebindDenial::ObservationTurn)?;
    let identity = turn.identity();
    match turn.admit_source(candidate) {
        Ok(_) => {}
        Err(crate::runtime::observation::UiObservationAdmissionDenial::DuplicateOwnerOrder) => {
            return Ok(NativeSourceAdmission::Duplicate(
                crate::runtime::rebind::UiDuplicateObservationReceipt::new(identity),
            ));
        }
        Err(crate::runtime::observation::UiObservationAdmissionDenial::HistoricalOwnerOrder) => {
            return Ok(NativeSourceAdmission::Superseded(
                crate::runtime::rebind::UiRebindSupersededReceipt::before_effects(
                    crate::runtime::rebind::UiRebindStoppedPhase::ObservationAdmission,
                ),
            ));
        }
        Err(denial) => {
            return Err(WorthUiNativeSourceRebindDenial::ObservationAdmission(
                denial,
            ));
        }
    }
    turn.seal()
        .map(NativeSourceAdmission::Admitted)
        .map_err(WorthUiNativeSourceRebindDenial::ObservationAdmission)
}

#[inline(never)]
fn classify_source_change(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    admitted: crate::runtime::observation::UiAdmittedObservationSet,
) -> Result<
    crate::runtime::observation::UiChangeClassificationOutcome,
    WorthUiNativeSourceRebindDenial,
> {
    session
        .classify_observations(admitted)
        .map_err(WorthUiNativeSourceRebindDenial::Classification)
}

#[inline(never)]
fn plan_source_rebind(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    classified: crate::runtime::observation::UiChangeClassificationOutcome,
    policy: crate::runtime::rebind::UiRebindExecutionPolicy,
) -> Result<NativeSourceRebindPlan, WorthUiNativeSourceRebindDenial> {
    let plan = match classified {
        crate::runtime::observation::UiChangeClassificationOutcome::ObservedNoChange(receipt) => {
            return Ok(NativeSourceRebindPlan::ObservedNoChange(receipt));
        }
        crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(change) => session
            .compile_preservation_rebind(change, policy)
            .map_err(WorthUiNativeSourceRebindDenial::Planning)?,
        crate::runtime::observation::UiChangeClassificationOutcome::Changed(change) => {
            let lifecycle = session
                .resolve_affected_scope(change)
                .map_err(WorthUiNativeSourceRebindDenial::Scope)?
                .resolve_identity_lifecycle()
                .map_err(WorthUiNativeSourceRebindDenial::Identity)?;
            session
                .compile_rebind_plan(lifecycle, policy)
                .map_err(WorthUiNativeSourceRebindDenial::Planning)?
        }
    };
    Ok(NativeSourceRebindPlan::Planned(plan))
}

#[inline(never)]
fn execute_source_rebind<'session>(
    session: &'session mut crate::facade::WorthUiActiveApplicationSession,
    plan: crate::runtime::rebind::UiRebindPlan,
    execution: crate::runtime::rebind::UiRebindExecutionRequest,
) -> Result<crate::runtime::rebind::UiRebindOutcome<'session>, WorthUiNativeSourceRebindDenial> {
    let now_tick = execution.now_tick();
    match session.prepare_rebind(plan, execution) {
        Ok(prepared) => Ok(prepared.execute(now_tick)),
        Err(crate::runtime::rebind::UiRebindPreparationDenial::TimedOutBeforeEffects) => Ok(
            crate::runtime::rebind::UiRebindOutcome::TimedOutBeforeEffects(
                crate::runtime::rebind::UiRebindTimeoutReceipt::elapsed(),
            ),
        ),
        Err(crate::runtime::rebind::UiRebindPreparationDenial::CancelledBeforeEffects) => Ok(
            crate::runtime::rebind::UiRebindOutcome::CancelledBeforeEffects(
                crate::runtime::rebind::UiRebindCancellationReceipt::cancelled(),
            ),
        ),
        Err(
            crate::runtime::rebind::UiRebindPreparationDenial::StaleSourceBasis
            | crate::runtime::rebind::UiRebindPreparationDenial::StalePredecessorGeneration,
        ) => Ok(
            crate::runtime::rebind::UiRebindOutcome::SupersededBeforeEffects(
                crate::runtime::rebind::UiRebindSupersededReceipt::before_effects(
                    crate::runtime::rebind::UiRebindStoppedPhase::FinalAdmission,
                ),
            ),
        ),
        Err(denial) => Err(WorthUiNativeSourceRebindDenial::Preparation(denial)),
    }
}
