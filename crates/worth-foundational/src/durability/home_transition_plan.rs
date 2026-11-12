//! Canonical recorded plan coordinates shared by Home and World owners.
use super::{
    CompositeAttemptAssociation, DurableProductBranchReference, HomeProgramTransitionIdentity,
    HomeProgramTransitionPlanDigest, HomeProgramTransitionStepAssociation,
    HomeProgramTransitionTargetMeaning,
};
use sha2::{Digest, Sha256};
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeProgramTransitionPlanDescription {
    identity: HomeProgramTransitionIdentity,
    predecessor: DurableProductBranchReference,
    digest: HomeProgramTransitionPlanDigest,
    steps: u32,
    target: HomeProgramTransitionTargetMeaning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HomeProgramTransitionDescriptionDenial {
    ForeignHome,
    StepOutOfRange,
    ForeignBranch,
}

impl HomeProgramTransitionPlanDescription {
    pub fn from_recorded(
        identity: HomeProgramTransitionIdentity,
        predecessor: DurableProductBranchReference,
        digest: HomeProgramTransitionPlanDigest,
        steps: u32,
        target: HomeProgramTransitionTargetMeaning,
    ) -> Result<Self, HomeProgramTransitionDescriptionDenial> {
        if identity.home() != predecessor.branch().owner().home() {
            return Err(HomeProgramTransitionDescriptionDenial::ForeignHome);
        }
        if target.step() >= steps {
            return Err(HomeProgramTransitionDescriptionDenial::StepOutOfRange);
        }
        Ok(Self {
            identity,
            predecessor,
            digest,
            steps,
            target,
        })
    }

    pub const fn identity(&self) -> HomeProgramTransitionIdentity {
        self.identity
    }
    pub fn predecessor(&self) -> &DurableProductBranchReference {
        &self.predecessor
    }
    pub const fn digest(&self) -> HomeProgramTransitionPlanDigest {
        self.digest
    }
    pub const fn steps(&self) -> u32 {
        self.steps
    }

    pub const fn target_meaning(&self) -> HomeProgramTransitionTargetMeaning {
        self.target
    }

    pub fn commitment(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth.home.transition.plan.v1\0");
        digest.update(self.identity.home().bytes());
        digest.update(self.identity.intent_position().get().to_le_bytes());
        let branch = self.predecessor.branch();
        digest.update(branch.owner().slot().get().to_le_bytes());
        digest.update((branch.name().len() as u64).to_le_bytes());
        digest.update(branch.name().as_bytes());
        digest.update(self.predecessor.incarnation().ordinal().to_le_bytes());
        digest.update(self.predecessor.generation().get().to_le_bytes());
        digest.update(self.predecessor.selected_commit().ordinal().to_le_bytes());
        digest.update(self.digest.bytes());
        digest.update(self.steps.to_le_bytes());
        digest.update(self.target.step().to_le_bytes());
        digest.update(self.target.catalog().bytes());
        digest.update(self.target.schema().bytes());
        digest.update(self.target.carrier().bytes());
        digest.update(
            self.target
                .whole_candidate_validation_version()
                .to_le_bytes(),
        );
        digest.finalize().into()
    }

    pub fn step_commitment(
        &self,
        step: HomeProgramTransitionStepAssociation,
        original: &CompositeAttemptAssociation,
    ) -> Result<[u8; 32], HomeProgramTransitionDescriptionDenial> {
        use HomeProgramTransitionDescriptionDenial as Denial;
        if step.transition() != self.identity || step.step() >= self.steps {
            return Err(Denial::StepOutOfRange);
        }
        if original.branch() != self.predecessor.branch()
            || original.incarnation() != self.predecessor.incarnation()
        {
            return Err(Denial::ForeignBranch);
        }
        let mut digest = Sha256::new();
        digest.update(b"worth.home.transition.step.v1\0");
        digest.update(self.commitment());
        digest.update(step.step().to_le_bytes());
        digest.update(original.attempt().ordinal().to_le_bytes());
        Ok(digest.finalize().into())
    }
}
