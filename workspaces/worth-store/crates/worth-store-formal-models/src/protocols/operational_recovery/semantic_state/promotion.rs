use super::{require_nonzero, Invariant, OperationalRecoverySemanticState, RejoinBinding};

impl OperationalRecoverySemanticState {
    pub(super) fn promotion_fence(
        &mut self,
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
        fence: [u8; 32],
        epoch: u64,
    ) -> Result<(), Invariant> {
        self.require_authorization(authorization_plan, execution_plan)?;
        require_nonzero(fence)?;
        if epoch == 0 {
            return Err(Invariant::PromotionEpochMonotonic);
        }
        self.promotion.execution_plan = Some(execution_plan);
        self.promotion.fence = Some(fence);
        self.promotion.epoch = Some(epoch);
        Ok(())
    }

    pub(super) fn promotion_record(
        &mut self,
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
        receipt: [u8; 32],
        fence: [u8; 32],
        epoch: u64,
    ) -> Result<(), Invariant> {
        self.require_authorization(authorization_plan, execution_plan)?;
        require_nonzero(receipt)?;
        if (
            self.promotion.execution_plan,
            self.promotion.fence,
            self.promotion.epoch,
        ) != (Some(execution_plan), Some(fence), Some(epoch))
        {
            return Err(Invariant::PromotionBindingPreserved);
        }
        self.promotion.receipt = Some(receipt);
        Ok(())
    }

    pub(super) fn promotion_publish(
        &mut self,
        receipt: [u8; 32],
        publication: [u8; 32],
        verification: [u8; 32],
        target: [u8; 32],
        epoch: u64,
    ) -> Result<(), Invariant> {
        for identity in [publication, verification, target] {
            require_nonzero(identity)?;
        }
        if (self.promotion.receipt, self.promotion.epoch) != (Some(receipt), Some(epoch)) {
            return Err(Invariant::PromotionBindingPreserved);
        }
        self.promotion.publication = Some(publication);
        Ok(())
    }

    pub(super) fn promotion_readmit(
        &self,
        publication: [u8; 32],
        serve_lease: [u8; 32],
        epoch: u64,
    ) -> Result<(), Invariant> {
        require_nonzero(serve_lease)?;
        if self.promotion.publication != Some(publication) {
            return Err(Invariant::PromotionBindingPreserved);
        }
        if self.promotion.epoch.is_none_or(|promoted| epoch < promoted) {
            return Err(Invariant::PromotionEpochMonotonic);
        }
        Ok(())
    }

    pub(super) fn rejoin_plan(
        &mut self,
        promotion_receipt: [u8; 32],
        plan: [u8; 32],
        disposition: u8,
    ) -> Result<(), Invariant> {
        require_nonzero(plan)?;
        if self.promotion.receipt != Some(promotion_receipt) || disposition > 2 {
            return Err(Invariant::RejoinBindingPreserved);
        }
        self.rejoin = Some(RejoinBinding { plan, disposition });
        Ok(())
    }

    pub(super) fn rejoin_complete(
        &self,
        plan: [u8; 32],
        receipt: [u8; 32],
        forensic: [u8; 32],
        target: [u8; 32],
        disposition: u8,
    ) -> Result<(), Invariant> {
        require_nonzero(receipt)?;
        let Some(binding) = self.rejoin else {
            return Err(Invariant::RejoinBindingPreserved);
        };
        if (binding.plan, binding.disposition) != (plan, disposition) {
            return Err(Invariant::RejoinBindingPreserved);
        }
        let complete = match disposition {
            0 => forensic != [0; 32] && target == [0; 32],
            1 => target == [0; 32],
            2 => forensic != [0; 32] && target != [0; 32],
            _ => false,
        };
        if !complete {
            return Err(Invariant::RejoinDispositionComplete);
        }
        Ok(())
    }
}
