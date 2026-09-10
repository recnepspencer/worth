use super::*;

impl WorthUiPresentationAsyncOwner {
    pub(super) fn retire_superseded_predecessors(
        &mut self,
        lineage: super::super::semantic_transition::PresentationLineageKey,
    ) -> Result<(), WorthUiPresentationSettlementDenial> {
        while let Some(key) = self
            .superseded_pending
            .iter()
            .find_map(|(key, pending)| (pending.lineage == lineage).then_some(*key))
        {
            let pending = self
                .superseded_pending
                .get(&key)
                .expect("selected superseded predecessor remains retained");
            let nonce = pending.nonce;
            let awaits_physical_completion = !pending.recovery_required;
            let completed = self.finish_superseded(key, false)?;
            if awaits_physical_completion {
                let replaced = self
                    .superseded_awaiting_completion
                    .insert((key, nonce), completed);
                assert!(
                    replaced.is_none(),
                    "one exact superseded receipt awaits one physical completion"
                );
            }
        }
        Ok(())
    }

    pub(super) fn finish_superseded(
        &mut self,
        key: PresentationAdmissionKey,
        stale_completion_observed: bool,
    ) -> Result<WorthUiPresentationPresentedReceipt, WorthUiPresentationSettlementDenial> {
        let pending = self
            .superseded_pending
            .remove(&key)
            .expect("validated superseded receipt remains retained");
        let observation = pending.admission.observation(&self.workspace).map_err(|_| {
            WorthUiPresentationSettlementDenial::Progress(
                WorthUiPresentationSettlementStop::QueryObservation,
            )
        });
        let observation = match observation {
            Ok(observation) => observation,
            Err(denial) => {
                self.superseded_pending.insert(key, pending);
                return Err(denial);
            }
        };
        if pending
            .admission
            .close_query_live_view(&mut self.workspace)
            .is_err()
        {
            self.superseded_pending.insert(key, pending);
            return Err(WorthUiPresentationSettlementDenial::Progress(
                WorthUiPresentationSettlementStop::QueryClose,
            ));
        }
        self.active_keys.remove(&key);
        if stale_completion_observed {
            self.record_transition(
                WorthUiPresentationTransitionKind::StaleCompletionRejected,
                key,
            );
        }
        Ok(WorthUiPresentationPresentedReceipt {
            observation,
            predecessor_observation: None,
        })
    }
}
