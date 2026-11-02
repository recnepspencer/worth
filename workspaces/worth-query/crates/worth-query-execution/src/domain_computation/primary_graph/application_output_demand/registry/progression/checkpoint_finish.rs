//! Ordinary checkpoint restoration returns custody whenever its claim moved.
use super::*;

pub(in crate::domain_computation::primary_graph) enum CheckpointFinishStop {
    Restored(WorthQueryOutputDemandDenial),
    Returned(WorthQueryOutputDemandDenial, WorthQueryOutputCheckpoint),
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_checkpoint_owned(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
        denial: Option<&WorthQueryOutputDemandDenial>,
    ) -> Result<(), CheckpointFinishStop> {
        use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind as Kind;
        let returned = |kind, checkpoint| {
            CheckpointFinishStop::Returned(WorthQueryOutputDemandDenial::new(kind, ""), checkpoint)
        };
        if !std::sync::Arc::ptr_eq(&self.state, &interest.owner.state) {
            return Err(returned(Kind::ForeignDemand, checkpoint));
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(mut record) = state.records.get_mut(&interest.key) else {
            return Err(returned(Kind::Closed, checkpoint));
        };
        let DemandState::Output(output) = &mut record.state else {
            return Err(CheckpointFinishStop::Returned(
                match &record.state {
                    DemandState::Failed(denial) => denial.clone(),
                    _ => WorthQueryOutputDemandDenial::new(Kind::SchedulingRejected, ""),
                },
                checkpoint,
            ));
        };
        if !output.owns_checkpoint(&checkpoint) || output.checkpoint.is_some() {
            return Err(returned(Kind::SchedulingRejected, checkpoint));
        }
        let stopped = match output.advancement() {
            WorthQueryOutputAdvancement::Claimed(active) if *active == claim => None,
            WorthQueryOutputAdvancement::Stopped {
                denial,
                interrupted_claim: Some(active),
            } if *active == claim => Some(denial.clone()),
            _ => return Err(returned(Kind::SchedulingRejected, checkpoint)),
        };
        output.install_checkpoint(checkpoint);
        let terminal_denial = stopped
            .as_ref()
            .or_else(|| denial.filter(|denial| super::super::required_stop::fails_row(denial)));
        let terminal = terminal_denial.is_some();
        if let Some(denial) = terminal_denial {
            output.finish_stopped(denial.clone());
        } else {
            output.reopen();
        }
        if terminal {
            record.release_obligations();
        }
        record.wake.notify();
        record.finish();
        let released_prerequisites =
            terminal.then(|| release_completed_prerequisites(&mut state, &interest.key));
        release_completed_member(&mut state, &interest.key);
        drop(state);
        drop(released_prerequisites);
        match stopped {
            Some(denial) => Err(CheckpointFinishStop::Restored(denial)),
            None => Ok(()),
        }
    }
}
