use super::*;

impl WorthUiPresentationAsyncOwner {
    pub fn resume_pending_admission(
        &mut self,
        receipt: &WorthUiPresentationIncompleteAdmission,
    ) -> Result<WorthUiPresentationPendingReceipt, WorthUiPresentationPendingAdmissionDenial> {
        if !correspondence::is_correspondence_authority(
            &self.correspondence_authority,
            &receipt.authority,
        ) {
            return Err(WorthUiPresentationPendingAdmissionDenial::ForeignCorrespondenceAuthority);
        }
        let key = PresentationAdmissionKey {
            attempt: receipt.attempt,
            binding: receipt.binding,
        };
        let pending = self
            .pending
            .remove(&key)
            .filter(|pending| pending.nonce == receipt.nonce)
            .ok_or(WorthUiPresentationPendingAdmissionDenial::NonPendingPosture)?;
        let observation = match pending.admission.observation(&self.workspace) {
            Ok(observation) => observation,
            Err(_) => {
                return Err(self.retain_admission_progress(
                    key,
                    pending,
                    WorthUiPresentationAdmissionStop::RuntimeObservation,
                ));
            }
        };
        self.finish_pending_admission(key, pending, observation)
    }

    pub(super) fn finish_pending_admission(
        &mut self,
        key: PresentationAdmissionKey,
        mut pending: PendingPresentationAdmission,
        observation: WorthUiPresentationAsyncObservation,
    ) -> Result<WorthUiPresentationPendingReceipt, WorthUiPresentationPendingAdmissionDenial> {
        if !pending.predecessor_supersession_complete {
            if !pending.reconstructing_unresolved_predecessor {
                if let Err(stop) =
                    self.supersede_pending_lineage(pending.lineage, &pending.admission)
                {
                    return Err(self.retain_admission_progress(key, pending, stop));
                }
            }
            pending.predecessor_supersession_complete = true;
        }
        let receipt = pending_receipt(&self.correspondence_authority, key, &pending, observation);
        if pending.superseding_pending_predecessor {
            self.record_transition(WorthUiPresentationTransitionKind::Superseded, key);
        } else if !pending.reconstructing_unresolved_predecessor {
            self.record_transition(WorthUiPresentationTransitionKind::Pending, key);
        }
        self.active_keys.insert(key);
        self.pending.insert(key, pending);
        Ok(receipt)
    }

    fn retain_admission_progress(
        &mut self,
        key: PresentationAdmissionKey,
        pending: PendingPresentationAdmission,
        stop: WorthUiPresentationAdmissionStop,
    ) -> WorthUiPresentationPendingAdmissionDenial {
        let receipt = admission_recovery(&self.correspondence_authority, key, &pending);
        self.active_keys.insert(key);
        self.pending.insert(key, pending);
        WorthUiPresentationPendingAdmissionDenial::AdmissionProgress(Box::new(receipt), stop)
    }

    pub(super) fn supersede_pending_lineage(
        &mut self,
        lineage: super::super::semantic_transition::PresentationLineageKey,
        displacing: &WorthUiPresentationRuntimeAdmission,
    ) -> Result<(), WorthUiPresentationAdmissionStop> {
        if let Some(key) = self.unresolved.iter().find_map(|(key, pending)| {
            (pending.lineage == lineage && pending.recovery_required).then_some(*key)
        }) {
            let mut pending = self
                .unresolved
                .remove(&key)
                .expect("selected unresolved lineage remains retained");
            let result = self.advance_pending_supersession(&mut pending, displacing);
            if pending.supersession_query_admitted {
                self.superseded_pending.insert(key, pending);
            } else {
                self.unresolved.insert(key, pending);
            }
            result?;
        }
        if let Some(key) = self
            .superseded_pending
            .iter()
            .find_map(|(key, pending)| (pending.lineage == lineage).then_some(*key))
        {
            let mut pending = self
                .superseded_pending
                .remove(&key)
                .expect("selected superseded lineage remains retained");
            let result = self.advance_pending_supersession(&mut pending, displacing);
            self.superseded_pending.insert(key, pending);
            result?;
        }
        let Some(key) = self
            .pending
            .iter()
            .find_map(|(key, pending)| (pending.lineage == lineage).then_some(*key))
        else {
            return Ok(());
        };
        let mut pending = self
            .pending
            .remove(&key)
            .expect("selected pending lineage remains retained");
        let result = self.advance_pending_supersession(&mut pending, displacing);
        if pending.supersession_query_admitted {
            self.superseded_pending.insert(key, pending);
        } else {
            self.pending.insert(key, pending);
        }
        result
    }

    fn advance_pending_supersession(
        &mut self,
        pending: &mut PendingPresentationAdmission,
        displacing: &WorthUiPresentationRuntimeAdmission,
    ) -> Result<(), WorthUiPresentationAdmissionStop> {
        if !pending.supersession_query_admitted {
            pending
                .admission
                .admit_supersession(&mut self.workspace, displacing)
                .map_err(|_| WorthUiPresentationAdmissionStop::QuerySupersession)?;
            pending.supersession_query_admitted = true;
        }
        if !pending.supersession_posture_observed {
            let observation = pending
                .admission
                .observation(&self.workspace)
                .map_err(|_| WorthUiPresentationAdmissionStop::RuntimeObservation)?;
            if observation.posture() != WorthUiPresentationAsyncPosture::Superseded {
                return Err(WorthUiPresentationAdmissionStop::UnexpectedSupersessionPosture);
            }
            pending.supersession_posture_observed = true;
        }
        Ok(())
    }
}

pub(super) fn pending_admission_complete(pending: &PendingPresentationAdmission) -> bool {
    pending.predecessor_supersession_complete
}

fn pending_receipt(
    authority: &std::sync::Arc<correspondence::PresentationCorrespondenceAuthority>,
    key: PresentationAdmissionKey,
    pending: &PendingPresentationAdmission,
    observation: WorthUiPresentationAsyncObservation,
) -> WorthUiPresentationPendingReceipt {
    WorthUiPresentationPendingReceipt {
        authority: std::sync::Arc::clone(authority),
        attempt: key.attempt,
        binding: key.binding,
        observation,
        nonce: pending.nonce,
    }
}

fn admission_recovery(
    authority: &std::sync::Arc<correspondence::PresentationCorrespondenceAuthority>,
    key: PresentationAdmissionKey,
    pending: &PendingPresentationAdmission,
) -> WorthUiPresentationIncompleteAdmission {
    WorthUiPresentationIncompleteAdmission {
        authority: std::sync::Arc::clone(authority),
        attempt: key.attempt,
        binding: key.binding,
        nonce: pending.nonce,
    }
}
