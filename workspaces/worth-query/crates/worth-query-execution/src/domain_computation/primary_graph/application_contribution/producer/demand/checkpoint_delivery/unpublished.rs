//! A stack loan restores original publication custody on refusal or unwinding.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    execution::unpublished::{PendingProducerPublication, RecoveredProducerPublication},
    ProducerInvocationAuthority,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};

/// The finish callback only moves the actual checkpoint under its registry
/// claim. It contains no authored code and never outlives this invocation.
struct PublicationLoan<Finish>
where
    Finish: FnOnce(Checkpoint) -> Result<(), WorthQueryOutputDemandDenial>,
{
    checkpoint: Option<Checkpoint>,
    finish: Option<Finish>,
    retired_owner: Option<PendingProducerPublication>,
    observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
    retired_carrier: Option<WorthQueryRecoveredProgramOutputSource>,
}
impl<Finish> PublicationLoan<Finish>
where
    Finish: FnOnce(Checkpoint) -> Result<(), WorthQueryOutputDemandDenial>,
{
    fn new(checkpoint: Checkpoint, finish: Finish) -> Self {
        Self {
            checkpoint: Some(checkpoint),
            finish: Some(finish),
            retired_owner: None,
            observation: None,
            retired_carrier: None,
        }
    }
    fn owner(&mut self) -> &mut PendingProducerPublication {
        let Some(Checkpoint::PublicationPending { owner, .. }) = self.checkpoint.as_mut() else {
            unreachable!("the publication loan holds its original pending checkpoint")
        };
        owner
    }
    fn performed(&mut self, performed: RecoveredProducerPublication) {
        let Some(Checkpoint::PublicationPending {
            owner,
            ready_backing,
        }) = self.checkpoint.take()
        else {
            unreachable!("one pending checkpoint owns its prepaid completion")
        };
        let RecoveredProducerPublication {
            receipt,
            change,
            observation,
            retired_carrier,
            original,
        } = performed;
        self.retired_owner = Some(owner);
        self.observation = Some(observation);
        self.retired_carrier = Some(retired_carrier);
        self.checkpoint = Some(Checkpoint::Published {
            receipt,
            delivery: Delivery::Change(change),
            ready_backing: ready_backing.bind_original_producing_publication(original),
        });
    }
    fn finish(mut self) -> Result<(), WorthQueryOutputDemandDenial> {
        let checkpoint = self
            .checkpoint
            .take()
            .expect("one original checkpoint handoff");
        let finish = self.finish.take().expect("one original registry claim");
        finish(checkpoint)
    }
}
impl<Finish> Drop for PublicationLoan<Finish>
where
    Finish: FnOnce(Checkpoint) -> Result<(), WorthQueryOutputDemandDenial>,
{
    fn drop(&mut self) {
        if let Some(checkpoint) = self.checkpoint.take() {
            if let Some(finish) = self.finish.take() {
                let _ = finish(checkpoint);
            }
        }
        // Fields retire after this restoration and outside all registry guards.
    }
}

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn close_unpublished_checkpoint(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        interest: &WorthQueryOutputDemandInterest,
        returned: &mut Option<Checkpoint>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputReadCloseDenial> {
        use crate::domain_computation::primary_graph::WorthQueryOutputReadCloseDenial as Denial;
        let Some((claim, checkpoint)) = self.output_demands.begin_unpublished_close(interest)?
        else {
            return Ok(());
        };
        let mut loan = PublicationLoan::new(checkpoint, |checkpoint| {
            use crate::domain_computation::primary_graph::application_output_demand::CheckpointFinishStop;
            match self
                .output_demands
                .finish_checkpoint_owned(interest, claim, checkpoint, None)
            {
                Ok(()) | Err(CheckpointFinishStop::Restored(_)) => Ok(()),
                Err(CheckpointFinishStop::Returned(stop, checkpoint)) => {
                    assert!(returned.is_none(), "one original checkpoint cleanup owner");
                    *returned = Some(checkpoint);
                    Err(stop)
                }
            }
        });
        let result = loan.owner().close(phase, self);
        loan.finish().map_err(Denial::Cleanup)?;
        result
    }

    pub(super) fn advance_unpublished_checkpoint(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        producer: &str,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
        owner: PendingProducerPublication,
        ready_backing: crate::domain_computation::primary_graph::application_output_demand::PreparedReadyBacking,
        finish: impl FnOnce(Checkpoint) -> Result<(), WorthQueryOutputDemandDenial>,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let mut loan = PublicationLoan::new(
            Checkpoint::PublicationPending {
                owner,
                ready_backing,
            },
            finish,
        );
        let entry = self
            .installed_producers
            .entries
            .get(producer)
            .expect("a claimed checkpoint retains its original installed producer");
        match entry.executor.recover_publication(
            phase,
            self,
            principal,
            request,
            invocation,
            entry.edition,
            loan.owner(),
        ) {
            Ok(Some(performed)) => {
                loan.performed(performed);
                loan.finish().map(|()| true)
            }
            Ok(None) => {
                loan.finish()?;
                Ok(false)
            }
            Err(stop) => {
                // Keep the row idle with its actual pending owner. The refusal
                // belongs to this invocation and cannot reschedule its producer.
                loan.finish()?;
                Err(stop)
            }
        }
    }
}
