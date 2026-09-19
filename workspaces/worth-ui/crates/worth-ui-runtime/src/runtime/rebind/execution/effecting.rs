use super::preparation::{UiPreparedRebind, UiPreparedRebindKind};

#[must_use = "effecting rebind authority must complete or be dropped to release capacity"]
pub struct UiEffectingRebind<'session> {
    plan: crate::runtime::rebind::UiRebindPlan,
    reservation: super::UiRebindReservation,
    kind: UiPreparedRebindKind<'session>,
    observations: crate::runtime::observation::UiEffectingObservationQueue,
}

#[must_use = "queued observations must be returned to the owning session"]
pub struct UiEffectingRebindCompletion<'session> {
    outcome: super::UiRebindOutcome<'session>,
    queued_observations: Box<[crate::runtime::observation::UiAdmittedObservationSet]>,
}

impl<'session> UiEffectingRebind<'session> {
    pub(super) fn begin(
        prepared: UiPreparedRebind<'session>,
    ) -> Result<Self, super::UiRebindDenialReceipt<'session>> {
        let UiPreparedRebind {
            plan,
            mut reservation,
            kind,
        } = prepared;
        if let Err(denial) = reservation.begin_effecting() {
            return Err(super::UiRebindDenialReceipt::capacity(
                denial,
                UiPreparedRebind {
                    plan,
                    reservation,
                    kind,
                },
            ));
        }
        let observations = crate::runtime::observation::UiEffectingObservationQueue::new(
            plan.effecting_observation_capacity(),
        );
        Ok(Self {
            plan,
            reservation,
            kind,
            observations,
        })
    }

    pub fn admit_observations(
        &mut self,
        set: crate::runtime::observation::UiAdmittedObservationSet,
    ) -> Result<
        crate::runtime::observation::UiEffectingObservationQueueAdmissionReceipt,
        crate::runtime::observation::UiEffectingObservationQueueCapacityStop,
    > {
        self.observations.admit(set)
    }

    pub fn queued_observation_count(&self) -> usize {
        self.observations.admitted_observation_count()
    }

    pub fn complete(self, now_tick: u64) -> UiEffectingRebindCompletion<'session> {
        let Self {
            mut plan,
            reservation,
            kind,
            observations,
        } = self;
        let outcome = match kind {
            UiPreparedRebindKind::Changed(replacement) => {
                let deadline = presentation_deadline(&plan);
                let outcome = replacement.present(deadline, now_tick);
                super::outcome::map_changed_first_attempt(plan, reservation, outcome)
            }
            UiPreparedRebindKind::Content(mut content) => {
                let preparation = content.refresh_before_effects(&mut plan);
                let deadline = presentation_deadline(&plan);
                let generation = plan.basis().candidate_generation().clone();
                let outcome = match preparation {
                    Ok(()) => content.present(deadline, now_tick),
                    Err(_) => crate::facade::entry::WorthUiMountedContentRebindOutcome::AdmissionDenied {
                        denial: crate::mounting::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged,
                        retry: content,
                    },
                };
                super::outcome::map_content_first_attempt(plan, reservation, generation, outcome)
            }
            UiPreparedRebindKind::EvidenceOnly(prepared) => match prepared.commit() {
                Ok((prior, active)) => {
                    match super::UiRebindReceipt::evidence_only(plan, reservation, prior, active) {
                        Ok(receipt) => super::UiRebindOutcome::Published(receipt),
                        Err(defect) => super::UiRebindOutcome::InternalDefect(defect),
                    }
                }
                Err(prepared) => super::UiRebindOutcome::RejectedBeforeEffects(
                    super::UiRebindDenialReceipt::prepared_basis_changed(UiPreparedRebind {
                        plan,
                        reservation,
                        kind: UiPreparedRebindKind::EvidenceOnly(*prepared),
                    }),
                ),
            },
        };
        UiEffectingRebindCompletion {
            outcome,
            queued_observations: observations.into_sets(),
        }
    }
}

impl<'session> UiEffectingRebindCompletion<'session> {
    pub fn outcome(&self) -> &super::UiRebindOutcome<'session> {
        &self.outcome
    }

    pub fn queued_observations(&self) -> &[crate::runtime::observation::UiAdmittedObservationSet] {
        &self.queued_observations
    }

    pub fn into_parts(
        self,
    ) -> (
        super::UiRebindOutcome<'session>,
        Box<[crate::runtime::observation::UiAdmittedObservationSet]>,
    ) {
        (self.outcome, self.queued_observations)
    }
}

fn presentation_deadline(
    plan: &crate::runtime::rebind::UiRebindPlan,
) -> worth_ui_host_contract::UiPresentationDeadline {
    let tick = match plan.execution_policy().deadline() {
        crate::runtime::rebind::UiRebindDeadlinePolicy::NoDeadline => u64::MAX,
        crate::runtime::rebind::UiRebindDeadlinePolicy::At(deadline) => deadline.tick(),
    };
    worth_ui_host_contract::UiPresentationDeadline::at_tick(tick)
}

#[cfg(test)]
#[path = "effecting_tests.rs"]
mod tests;
