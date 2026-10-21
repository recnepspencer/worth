//! One prepaid finish for an exact selected Published or Delivered claim.

use std::sync::Arc;

use super::super::required_work::RequiredWorkMembership;
use super::*;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

/// A stopped claim may already have restored its checkpoint. A moved claim
/// returns the original value to the successor demand for exact retry.
pub(in crate::domain_computation::primary_graph) enum SelectedCheckpointFinishStop {
    Restored(WorthQueryOutputDemandDenial),
    Returned(WorthQueryOutputDemandDenial, WorthQueryOutputCheckpoint),
}

pub(in crate::domain_computation::primary_graph) struct PreparedSelectedCheckpointFinish<'a> {
    registry: &'a WorthQueryOutputDemandRegistry,
    interest: &'a WorthQueryOutputDemandInterest,
    member: Arc<RequiredWorkMembership>,
}

impl WorthQueryOutputDemandRegistry {
    /// Prepare before the registry's begin. No later budget refusal may leave
    /// a claimed checkpoint outside both the row and successor custody.
    pub(in crate::domain_computation::primary_graph) fn prepare_selected_checkpoint_finish<'a>(
        &'a self,
        interest: &'a WorthQueryOutputDemandInterest,
    ) -> Result<PreparedSelectedCheckpointFinish<'a>, WorthQueryOutputDemandDenial> {
        if !Arc::ptr_eq(&self.state, &interest.owner.state) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "",
            ));
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Pin the original required membership before moving its checkpoint.
        let record = state
            .records
            .get(&interest.key)
            .expect("live selected Interest retains its record");
        let member = record
            .work_membership
            .as_ref()
            .cloned()
            .ok_or_else(|| empty_denial(WorthQueryOutputDemandDenialKind::ForeignDemand))?;
        Ok(PreparedSelectedCheckpointFinish {
            registry: self,
            interest,
            member,
        })
    }
}

impl PreparedSelectedCheckpointFinish<'_> {
    /// Claim only a real Published/Delivered checkpoint. Scheduled and
    /// already Ready successors are left untouched for their own owners.
    pub(in crate::domain_computation::primary_graph) fn claim(
        self,
    ) -> Result<
        Option<(
            Self,
            WorthQueryOutputClaimIdentity,
            WorthQueryOutputCheckpoint,
        )>,
        WorthQueryOutputDemandDenial,
    > {
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(mut record) = state.records.get_mut(&self.interest.key) else {
            return Ok(None);
        };
        if !record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.member))
            || !matches!(&record.state,
                DemandState::Output(output)
                    if matches!(output.advancement(), WorthQueryOutputAdvancement::Idle)
                        && matches!(&output.checkpoint,
                            Some(WorthQueryOutputCheckpoint::Published { .. }
                                | WorthQueryOutputCheckpoint::Delivered { .. })))
        {
            return Ok(None);
        }
        let next = super::begin_record(&mut record, "");
        record.finish();
        drop(state);
        match next {
            WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } => {
                Ok(Some((self, claim, checkpoint)))
            }
            WorthQueryOutputDemandAdvanceAdmission::Failed(denial) => Err(denial),
            _ => unreachable!("selected checkpoint precheck admitted only Output Idle"),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        claim: WorthQueryOutputClaimIdentity,
        checkpoint: WorthQueryOutputCheckpoint,
        denial: Option<&WorthQueryOutputDemandDenial>,
    ) -> Result<(), SelectedCheckpointFinishStop> {
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(mut record) = state.records.get_mut(&self.interest.key) else {
            return Err(SelectedCheckpointFinishStop::Returned(
                empty_denial(WorthQueryOutputDemandDenialKind::Closed),
                checkpoint,
            ));
        };
        if !record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.member))
        {
            return Err(SelectedCheckpointFinishStop::Returned(
                empty_denial(WorthQueryOutputDemandDenialKind::SchedulingRejected),
                checkpoint,
            ));
        }
        let DemandState::Output(output) = &mut record.state else {
            return Err(SelectedCheckpointFinishStop::Returned(
                empty_denial(WorthQueryOutputDemandDenialKind::SchedulingRejected),
                checkpoint,
            ));
        };
        match output.advancement() {
            WorthQueryOutputAdvancement::Stopped {
                denial: stopped,
                interrupted_claim: Some(active),
            } if *active == claim => {
                let stopped = retained_denial(stopped);
                if output.checkpoint.is_some() {
                    return Err(SelectedCheckpointFinishStop::Returned(stopped, checkpoint));
                }
                output.install_checkpoint(checkpoint);
                output.finish_stopped(retained_denial(&stopped));
                record.wake.notify();
                record.finish();
                state.defer_terminal_cleanup(self.member.key_arc());
                drop(state);
                Err(SelectedCheckpointFinishStop::Restored(stopped))
            }
            WorthQueryOutputAdvancement::Claimed(active) if *active == claim => {
                output.install_checkpoint(checkpoint);
                // The shared required row stops only for a stop intrinsic to
                // it; any other stays with the advance that met it, and the
                // row's checkpoint waits for a later claim.
                let terminal = denial.is_some_and(super::super::required_stop::fails_row);
                if terminal {
                    output.finish_stopped(retained_denial(denial.expect("terminal cause exists")));
                } else {
                    output.reopen();
                }
                record.wake.notify();
                record.finish();
                if terminal {
                    // Existing admitted cleanup destroys variable obligations
                    // and refunds their final owner after this guarded step.
                    state.defer_terminal_cleanup(self.member.key_arc());
                }
                drop(state);
                Ok(())
            }
            _ => Err(SelectedCheckpointFinishStop::Returned(
                empty_denial(WorthQueryOutputDemandDenialKind::SchedulingRejected),
                checkpoint,
            )),
        }
    }
}

fn retained_denial(source: &WorthQueryOutputDemandDenial) -> WorthQueryOutputDemandDenial {
    let mut retained = empty_denial(source.kind());
    retained.recovery_posture = source.recovery_posture();
    retained
}

fn empty_denial(kind: WorthQueryOutputDemandDenialKind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "")
}
