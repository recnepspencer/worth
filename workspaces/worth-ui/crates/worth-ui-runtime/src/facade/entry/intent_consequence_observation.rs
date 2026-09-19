pub(super) struct WorthUiPreparedIntentConsequenceObservation {
    pub(super) set: crate::runtime::observation::UiAdmittedObservationSet,
    pub(super) progress: WorthUiClosedConsequenceObservation,
    pub(super) posture: Option<crate::mounting::UiIntentPostureCommit>,
    pub(super) admitted_count: usize,
}

/// Exact closed owner views retained alongside deferred observation progress.
pub(super) struct WorthUiClosedConsequenceObservation {
    progress: crate::runtime::observation::UiPreparedObservationProgressCommit,
    pub(super) appearance: Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    pub(super) pointer: Option<crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
    predecessor_appearance: Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    predecessor_pointer: Option<crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot>,
}

pub(super) struct WorthUiPreparedConsequenceObservationCommit {
    closed: WorthUiClosedConsequenceObservation,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
}

impl WorthUiPreparedConsequenceObservationCommit {
    pub(super) fn appearance(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.closed.appearance.as_ref()
    }

    pub(super) fn validate(
        &self,
        session: &super::WorthUiActiveApplicationSession,
        frame: &crate::mounting::UiPreparedMountedFrame,
    ) -> Result<(), crate::runtime::rebind::UiRebindPreparationDenial> {
        use crate::runtime::rebind::UiRebindPreparationDenial as Denial;
        if self.frame != frame.canonical_core().frame() {
            return Err(Denial::ConsequenceFrameMismatch);
        }
        self.validate_owner_predecessor(session)
    }

    pub(super) fn validate_owner_predecessor(
        &self,
        session: &super::WorthUiActiveApplicationSession,
    ) -> Result<(), crate::runtime::rebind::UiRebindPreparationDenial> {
        use crate::runtime::rebind::UiRebindPreparationDenial as Denial;
        let appearance_matches = match (
            self.closed.predecessor_appearance.as_ref(),
            session.appearance_owner_snapshot.as_ref(),
        ) {
            (None, None) => true,
            (Some(before), Some(current)) => current
                .refresh_receipt_sources(
                    session.focus.as_ref(),
                    session.selection.as_ref(),
                    &session.intent_admission,
                    &session.intent_application_facts,
                    &session.interaction,
                )
                .is_some_and(|current| before.same_publication_predecessor(&current)),
            _ => false,
        };
        let pointer_matches = match (
            self.closed.predecessor_pointer.as_ref(),
            session.pointer_affordance_snapshot.as_ref(),
        ) {
            (None, None) => true,
            (Some(before), Some(current)) => before.same_owner_snapshot(current),
            _ => false,
        };
        if !appearance_matches || !pointer_matches {
            return Err(Denial::StaleConsequenceOwnerSnapshot);
        }
        Ok(())
    }
}

impl super::WorthUiActiveApplicationSession {
    pub(super) fn commit_consequence_observation(
        &mut self,
        prepared: WorthUiPreparedConsequenceObservationCommit,
    ) {
        let prepared = prepared.closed;
        self.application
            .commit_prepared_observation_progress(prepared.progress);
        self.appearance_owner_snapshot = prepared.appearance;
        self.pointer_affordance_snapshot = prepared.pointer;
    }

    pub(super) fn prepare_observed_intent_consequence_frame(
        &mut self,
        content: crate::mounting::UiMountedSemanticContentInput,
        overlay_revision: u64,
        overlays: Vec<crate::mounting::UiMountedPortalOverlayProjectionInput>,
        mut closed: WorthUiClosedConsequenceObservation,
    ) -> Result<
        (
            crate::mounting::UiPreparedMountedFrame,
            WorthUiPreparedConsequenceObservationCommit,
        ),
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let frame = self.prepare_closed_owner_content_frame(
            content,
            overlay_revision,
            overlays,
            &mut closed,
        )?;
        let prepared = WorthUiPreparedConsequenceObservationCommit {
            frame: frame.canonical_core().frame(),
            closed,
        };
        prepared.validate(self, &frame)?;
        Ok((frame, prepared))
    }
}

pub(super) struct WorthUiIntentConsequenceObservationStop {
    pub(super) reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
    pub(super) batch: Box<crate::runtime::observation::UiIntentConsequenceObservationBatch>,
}

pub(super) fn prepare_intent_consequence_observation(
    session: &mut super::WorthUiActiveApplicationSession,
    batch: crate::runtime::observation::UiIntentConsequenceObservationBatch,
) -> Result<WorthUiPreparedIntentConsequenceObservation, WorthUiIntentConsequenceObservationStop> {
    // Native input may have advanced owner receipts since the last painted
    // frame. Seal the current predecessor that validation will later compare,
    // without publishing those facts or replacing the accepted snapshot.
    let predecessor_appearance = session
        .appearance_owner_snapshot
        .as_ref()
        .and_then(|snapshot| {
            snapshot.refresh_receipt_sources(
                session.focus.as_ref(),
                session.selection.as_ref(),
                &session.intent_admission,
                &session.intent_application_facts,
                &session.interaction,
            )
        });
    let predecessor_pointer = session.pointer_affordance_snapshot.clone();
    let mut turn = match session.begin_observation_turn() {
        Ok(turn) => turn,
        Err(denial) => {
            return Err(WorthUiIntentConsequenceObservationStop {
                reason:
                    crate::runtime::intent_execution::UiIntentConsequenceStopReason::ObservationTurn(
                        denial,
                    ),
                batch: Box::new(batch),
            });
        }
    };
    let admission = match turn.admit_intent_consequence_batch(batch) {
        Ok(admission) => admission,
        Err(stop) => {
            drop(turn);
            let (reason, batch) = stop.into_parts();
            return Err(WorthUiIntentConsequenceObservationStop {
                reason: map_observation_stop(reason),
                batch,
            });
        }
    };
    let admitted_count = admission.admitted().len();
    let posture = admission.into_posture_commit();
    let (mut set, progress) = turn
        .prepare_seal()
        .expect("successful nonempty consequence admission prepares one deferred seal");
    let progress = WorthUiClosedConsequenceObservation {
        progress,
        appearance: set.take_appearance_owner_snapshot(),
        pointer: set.take_pointer_snapshot(),
        predecessor_appearance,
        predecessor_pointer,
    };
    Ok(WorthUiPreparedIntentConsequenceObservation {
        set,
        progress,
        posture,
        admitted_count,
    })
}

fn map_observation_stop(
    reason: crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason,
) -> crate::runtime::intent_execution::UiIntentConsequenceStopReason {
    match reason {
        crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason::Observation(
            denial,
        ) => crate::runtime::intent_execution::UiIntentConsequenceStopReason::ObservationAdmission(
            denial,
        ),
        crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason::Query(
            denial,
        ) => {
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::QueryAdmission(denial)
        }
    }
}
