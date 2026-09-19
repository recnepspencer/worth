use super::partition::SignalConditionalTemporalState;
use super::SignalConditionalTemporalPartitionDenial;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SignalTemporalPromotionBoundary {
    AfterClockAdvance,
    AfterFirstPromotion,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum SignalTemporalPromotionFaultAction {
    Diverge,
    Unwind,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SignalTemporalPromotionFault {
    pub(super) boundary: SignalTemporalPromotionBoundary,
    pub(super) action: SignalTemporalPromotionFaultAction,
}

impl SignalConditionalTemporalState {
    pub(super) fn reach_promotion_fault_boundary(
        &mut self,
        boundary: SignalTemporalPromotionBoundary,
    ) -> Result<(), SignalConditionalTemporalPartitionDenial> {
        let Some(fault) = self
            .promotion_fault
            .filter(|fault| fault.boundary == boundary)
        else {
            return Ok(());
        };
        self.promotion_fault = None;
        match fault.action {
            SignalTemporalPromotionFaultAction::Diverge => {
                Err(SignalConditionalTemporalPartitionDenial::TemporalOperation(
                    crate::data::error::SignalError::invalid_input(
                        "injected internal temporal promotion divergence",
                    ),
                ))
            }
            SignalTemporalPromotionFaultAction::Unwind => {
                panic!("injected temporal promotion unwind at {boundary:?}")
            }
        }
    }
}
