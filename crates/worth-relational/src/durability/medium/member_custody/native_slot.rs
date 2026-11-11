//! Inline custody of one original Native-owned physical registration.
use super::*;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(crate) enum NativeMemberRetirementSlot {
    Original(RelationalDurableMemberCustody),
    Retiring(RelationalDurableMemberRetirement),
    Loaned,
    Retired {
        _original: RetiredRelationalDurableMemberCustody,
    },
}

#[derive(Debug)]
pub(crate) enum NativeMemberSlotDenial {
    Busy,
    Refused(RelationalDurableMemberRetirementRefusal),
}

/// The close cursor owns this before releasing the exact containing slot lock.
/// A completed callback observation stays beside its original task until return.
pub(crate) struct NativeMemberRetirementHandoff {
    task: RelationalDurableMemberRetirement,
    observation: Option<RelationalDurableMemberRetirementObservation>,
}

impl NativeMemberRetirementSlot {
    pub(crate) fn retain_serving(&self) -> Self {
        match self {
            Self::Original(token) => Self::Original(token.clone()),
            Self::Retiring(_) | Self::Loaned | Self::Retired { .. } => {
                panic!("retired Native custody cannot create a speculative owner")
            }
        }
    }
    /// No provider call or allocation occurs while the original slot is held.
    pub(crate) fn loan(
        &mut self,
    ) -> Result<Option<NativeMemberRetirementHandoff>, NativeMemberSlotDenial> {
        if matches!(self, Self::Retired { .. }) {
            return Ok(None);
        }
        if matches!(self, Self::Loaned) {
            return Err(NativeMemberSlotDenial::Busy);
        }
        let task = match std::mem::replace(self, Self::Loaned) {
            Self::Original(token) => token.into_retirement(),
            Self::Retiring(task) => task,
            Self::Loaned | Self::Retired { .. } => unreachable!("actual live slot checked"),
        };
        Ok(Some(NativeMemberRetirementHandoff {
            task,
            observation: None,
        }))
    }
    pub(crate) fn return_handoff(
        &mut self,
        handoff: NativeMemberRetirementHandoff,
    ) -> RelationalDurableMemberRetirementObservation {
        assert!(
            matches!(self, Self::Loaned),
            "same original slot remains loaned"
        );
        let NativeMemberRetirementHandoff { task, observation } = handoff;
        let observation = observation.expect("callback observation retained before slot return");
        *self = if matches!(
            observation,
            RelationalDurableMemberRetirementObservation::Complete
        ) {
            Self::Retired {
                _original: task
                    .into_retired()
                    .unwrap_or_else(|_| unreachable!("owner reported Complete")),
            }
        } else {
            Self::Retiring(task)
        };
        observation
    }
}

impl NativeMemberRetirementHandoff {
    pub(crate) fn observe(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) {
        if self.observation.is_none() {
            self.observation = Some(self.task.advance(request, source));
        }
    }
}
