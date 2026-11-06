//! Refuse orphaning this interest's unfinished native publication.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputReadCloseDenial;
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn begin_unpublished_close(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> Result<
        Option<(WorthQueryOutputClaimIdentity, WorthQueryOutputCheckpoint)>,
        WorthQueryOutputReadCloseDenial,
    > {
        if !std::sync::Arc::ptr_eq(&self.state, &interest.owner.state) {
            return Err(WorthQueryOutputReadCloseDenial::ForeignDemand);
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(mut record) = state.records.get_mut(&interest.key) else {
            return Ok(None);
        };
        if !matches!(&record.state, DemandState::Output(output)
            if matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::PublicationPending { .. })))
        {
            return Ok(None);
        }
        if let DemandState::Output(output) = &mut record.state
            && matches!(
                output.advancement(),
                WorthQueryOutputAdvancement::Stopped { .. }
            )
        {
            let claimed = output.claim_stopped_checkpoint_for_close();
            record.finish();
            return claimed
                .map(Some)
                .ok_or(WorthQueryOutputReadCloseDenial::NativePublicationRetained);
        }
        match progression::begin_record(&mut record, "") {
            WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } => {
                Ok(Some((claim, checkpoint)))
            }
            WorthQueryOutputDemandAdvanceAdmission::Failed(stop) => {
                Err(WorthQueryOutputReadCloseDenial::Cleanup(stop))
            }
            _ => Err(WorthQueryOutputReadCloseDenial::NativePublicationRetained),
        }
    }
}
impl WorthQueryOutputDemandInterest {
    pub fn require_read_closeable(&self) -> Result<(), WorthQueryOutputReadCloseDenial> {
        let state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = state.records.get(&self.key) else {
            return Ok(());
        };
        if record.interests == 1
            && record.held_successor.is_none()
            && matches!(&record.state, DemandState::Output(output)
                if matches!(&output.checkpoint, Some(WorthQueryOutputCheckpoint::Published { .. } | WorthQueryOutputCheckpoint::Delivered { .. }))
                    || matches!(&output.checkpoint, Some(WorthQueryOutputCheckpoint::PublicationPending { owner, .. }) if !owner.is_closed())
                    || matches!(output.advancement(), WorthQueryOutputAdvancement::Claimed(_)
                        | WorthQueryOutputAdvancement::Stopped { interrupted_claim: Some(_), .. }))
        {
            return Err(WorthQueryOutputReadCloseDenial::NativePublicationRetained);
        }
        Ok(())
    }
}
