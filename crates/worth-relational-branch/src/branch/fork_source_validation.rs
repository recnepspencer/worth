use std::sync::Arc;

use crate::branch::fork_source_basis::RelationalForkSourceDescriptor;
use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::root::RelationalBranchRoot;
use crate::branch::target::RelationalBranchTarget;
use crate::branch::transition::RelationalBranchTransition;
use crate::branch::version::RelationalBranchVersion;
use crate::history::data::BranchId;

pub enum ForkSourceInput {
    Current(crate::branch::fork_source_basis::AdmittedRelationalForkSourceBasis),
    Retained(crate::branch::retained_fork_source::AdmittedRelationalRetainedForkSource),
    /// Recovery replay of a recorded retained fork: the historical source root
    /// was rebuilt from durable history and verified against `source_head`.
    Recovered(Box<ValidatedForkSource>),
}

/// How the fork source's immutable root is held until installation.
pub enum ForkSourceCustody {
    /// The source branch's current head, rechecked inside the reservation.
    Current,
    /// An original admitted retained basis; its semantic retention obligation
    /// lives until installation.
    Retained(crate::branch::basis::AdmittedRelationalBranchBasis),
    /// A recorded retained fork replayed from durable history.
    Recovered,
}

impl ForkSourceCustody {
    pub const fn is_retained(&self) -> bool {
        !matches!(self, Self::Current)
    }
}

/// A source admitted by its actual current or retained receiver.
pub struct ValidatedForkSource {
    pub descriptor: RelationalForkSourceDescriptor,
    pub source_observation: RelationalBranchReferenceObservation,
    pub source_truth_version: RelationalBranchVersion,
    pub source_root: Arc<RelationalBranchRoot>,
    /// The source's selected head in the canonical runtime frame.
    pub source_head: RelationalBranchTarget,
    pub custody: ForkSourceCustody,
}

impl ValidatedForkSource {
    /// The canonical transition that forks `target` from this source.
    pub fn transition(&self, target: BranchId) -> RelationalBranchTransition {
        if self.custody.is_retained() {
            return RelationalBranchTransition::RetainedFork {
                target,
                source: self.descriptor.source_branch().clone(),
                source_generation: self.source_observation.generation(),
                source_truth_version: self.source_truth_version,
                source_head: self.source_head.clone(),
            };
        }
        RelationalBranchTransition::Fork {
            target,
            source: self.descriptor.source_branch().clone(),
            source_generation: self.source_observation.generation(),
            source_truth_version: self.source_truth_version,
            source_head: self.source_head.clone(),
        }
    }
}
