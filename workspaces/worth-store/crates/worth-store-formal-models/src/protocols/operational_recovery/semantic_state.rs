use std::collections::BTreeMap;

use super::{
    binding::{OperationalRecoveryActionBinding as Binding, PublicationBinding},
    OperationalRecoveryAction, OperationalRecoveryInvariant as Invariant,
};

mod promotion;

#[derive(Debug, Default)]
pub(super) struct OperationalRecoverySemanticState {
    authorizations: BTreeMap<[u8; 32], Option<[u8; 32]>>,
    publication: Option<PublicationBinding>,
    bootstrap: Option<BootstrapBinding>,
    promotion: PromotionBinding,
    rejoin: Option<RejoinBinding>,
}

#[derive(Debug, Clone, Copy)]
struct BootstrapBinding {
    receipt: [u8; 32],
    source_lease: [u8; 32],
}

#[derive(Debug, Clone, Copy, Default)]
struct PromotionBinding {
    execution_plan: Option<[u8; 32]>,
    fence: Option<[u8; 32]>,
    receipt: Option<[u8; 32]>,
    publication: Option<[u8; 32]>,
    epoch: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
struct RejoinBinding {
    plan: [u8; 32],
    disposition: u8,
}

impl OperationalRecoverySemanticState {
    pub(super) fn apply(&mut self, action: &OperationalRecoveryAction) -> Result<(), Invariant> {
        match &action.binding {
            Binding::None => Ok(()),
            Binding::Authorization {
                plan,
                execution,
                replayed,
            } => self.authorization(*plan, *execution, *replayed),
            Binding::PublicationPrepared(binding) => self.prepare_publication(*binding),
            Binding::PublicationPending(binding) => self.require_publication(*binding),
            Binding::PublicationDisposition {
                publication,
                observed_authority,
            } => self.disposition(
                *publication,
                *observed_authority,
                action.authority_identity(),
            ),
            Binding::FenceReleased {
                publication,
                fence,
                fence_plan,
            } => self.release_fence(*publication, *fence, *fence_plan),
            Binding::BootstrapTransfer {
                authorization_plan,
                execution_plan,
                receipt,
                source_lease,
                target,
            } => self.bootstrap_transfer(
                *authorization_plan,
                *execution_plan,
                *receipt,
                *source_lease,
                *target,
            ),
            Binding::BootstrapCompleted {
                receipt,
                source_lease,
                verification,
            } => self.bootstrap_complete(*receipt, *source_lease, *verification),
            Binding::PromotionFence {
                authorization_plan,
                execution_plan,
                fence,
                epoch,
            } => self.promotion_fence(*authorization_plan, *execution_plan, *fence, *epoch),
            Binding::PromotionRecorded {
                authorization_plan,
                execution_plan,
                receipt,
                fence,
                epoch,
            } => self.promotion_record(
                *authorization_plan,
                *execution_plan,
                *receipt,
                *fence,
                *epoch,
            ),
            Binding::PromotionPublished {
                receipt,
                publication,
                verification,
                target,
                epoch,
            } => self.promotion_publish(*receipt, *publication, *verification, *target, *epoch),
            Binding::PromotionReadmitted {
                publication,
                serve_lease,
                epoch,
            } => self.promotion_readmit(*publication, *serve_lease, *epoch),
            Binding::RejoinPlanned {
                promotion_receipt,
                plan,
                disposition,
            } => self.rejoin_plan(*promotion_receipt, *plan, *disposition),
            Binding::RejoinCompleted {
                plan,
                receipt,
                forensic_retention,
                rebootstrap_target,
                disposition,
            } => self.rejoin_complete(
                *plan,
                *receipt,
                *forensic_retention,
                *rebootstrap_target,
                *disposition,
            ),
        }
    }

    fn authorization(
        &mut self,
        plan: [u8; 32],
        execution: Option<[u8; 32]>,
        replayed: bool,
    ) -> Result<(), Invariant> {
        require_nonzero(plan)?;
        if replayed || self.authorizations.contains_key(&plan) {
            return Err(Invariant::AuthorizationReplayRejected);
        }
        if execution == Some([0; 32]) {
            return Err(Invariant::SemanticIdentityNonZero);
        }
        self.authorizations.insert(plan, execution);
        Ok(())
    }

    fn prepare_publication(&mut self, binding: PublicationBinding) -> Result<(), Invariant> {
        require_nonzero(binding.publication())?;
        require_nonzero(binding.fence())?;
        require_nonzero(binding.fence_plan())?;
        self.publication = Some(binding);
        Ok(())
    }

    fn require_publication(&mut self, binding: PublicationBinding) -> Result<(), Invariant> {
        if self.publication != Some(binding) {
            return Err(Invariant::PublicationBindingPreserved);
        }
        Ok(())
    }

    fn disposition(
        &self,
        publication: [u8; 32],
        observed_authority: [u8; 32],
        expected_authority: [u8; 32],
    ) -> Result<(), Invariant> {
        let Some(binding) = self.publication else {
            return Err(Invariant::PublicationBindingPreserved);
        };
        if binding.publication() != publication || observed_authority != expected_authority {
            return Err(Invariant::PublicationBindingPreserved);
        }
        Ok(())
    }

    fn release_fence(
        &self,
        publication: [u8; 32],
        fence: [u8; 32],
        fence_plan: [u8; 32],
    ) -> Result<(), Invariant> {
        let Some(binding) = self.publication else {
            return Err(Invariant::PublicationBindingPreserved);
        };
        if (binding.publication(), binding.fence(), binding.fence_plan())
            != (publication, fence, fence_plan)
        {
            return Err(Invariant::PublicationBindingPreserved);
        }
        Ok(())
    }

    fn bootstrap_transfer(
        &mut self,
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
        receipt: [u8; 32],
        source_lease: [u8; 32],
        target: [u8; 32],
    ) -> Result<(), Invariant> {
        self.require_authorization(authorization_plan, execution_plan)?;
        for identity in [receipt, source_lease, target] {
            require_nonzero(identity)?;
        }
        self.bootstrap = Some(BootstrapBinding {
            receipt,
            source_lease,
        });
        Ok(())
    }

    fn bootstrap_complete(
        &self,
        receipt: [u8; 32],
        source_lease: [u8; 32],
        verification: [u8; 32],
    ) -> Result<(), Invariant> {
        require_nonzero(verification)?;
        let Some(binding) = self.bootstrap else {
            return Err(Invariant::BootstrapBindingPreserved);
        };
        if (binding.receipt, binding.source_lease) != (receipt, source_lease) {
            return Err(Invariant::BootstrapBindingPreserved);
        }
        Ok(())
    }

    pub(super) fn require_authorization(
        &self,
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
    ) -> Result<(), Invariant> {
        if self
            .authorizations
            .get(&authorization_plan)
            .is_none_or(|expected| expected.is_some_and(|value| value != execution_plan))
        {
            return Err(Invariant::AuthorizationPlanBindingPreserved);
        }
        Ok(())
    }
}

pub(super) fn require_nonzero(identity: [u8; 32]) -> Result<(), Invariant> {
    if identity == [0; 32] {
        Err(Invariant::SemanticIdentityNonZero)
    } else {
        Ok(())
    }
}
