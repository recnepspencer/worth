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
}

/// A source admitted by its actual current or retained receiver.
pub struct ValidatedForkSource {
    pub descriptor: RelationalForkSourceDescriptor,
    pub source_observation: RelationalBranchReferenceObservation,
    pub source_truth_version: RelationalBranchVersion,
    pub source_root: Arc<RelationalBranchRoot>,
    /// The source's selected head in the canonical runtime frame.
    pub source_head: RelationalBranchTarget,
    // Keep the original semantic retention obligation until installation.
    pub retained: Option<crate::branch::basis::AdmittedRelationalBranchBasis>,
}

impl ValidatedForkSource {
    /// The canonical transition that forks `target` from this source.
    pub fn transition(&self, target: BranchId) -> RelationalBranchTransition {
        if self.retained.is_some() {
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
