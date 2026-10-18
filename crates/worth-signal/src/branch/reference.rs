use worth_foundational::{
    FoundationalBranchComparisonBasis, FoundationalBranchForkBasis,
    FoundationalBranchReferenceGeneration, FoundationalBranchReferenceObservation,
    FoundationalBranchTarget,
};

use super::identity::{signal_branch_identity, SignalBranchIdentityConstructionDenial};
use super::target::{SignalBranchTarget, SignalBranchTargetConstructionDenial};

pub type SignalBranchObservation = FoundationalBranchReferenceObservation<SignalBranchTarget>;
pub type SignalBranchForkBasis = FoundationalBranchForkBasis<SignalBranchTarget>;
pub type SignalBranchComparisonBasis = FoundationalBranchComparisonBasis<SignalBranchTarget>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBranchObservationConstructionDenial {
    InvalidIdentity(SignalBranchIdentityConstructionDenial),
    InvalidTarget(SignalBranchTargetConstructionDenial),
    GraphInstanceMismatch {
        observation_graph_instance_id: String,
        target_graph_instance_id: String,
    },
}

pub fn signal_branch_observation(
    graph_instance_id: impl AsRef<str>,
    branch_id: u64,
    branch_name: impl AsRef<str>,
    target: FoundationalBranchTarget<SignalBranchTarget>,
    generation: FoundationalBranchReferenceGeneration,
) -> Result<SignalBranchObservation, SignalBranchObservationConstructionDenial> {
    let graph_instance_id = graph_instance_id.as_ref();
    if let FoundationalBranchTarget::Basis(target) = &target
        && target.graph_instance_id() != graph_instance_id
    {
        return Err(
            SignalBranchObservationConstructionDenial::GraphInstanceMismatch {
                observation_graph_instance_id: graph_instance_id.to_owned(),
                target_graph_instance_id: target.graph_instance_id().to_owned(),
            },
        );
    }
    let branch_identity = signal_branch_identity(graph_instance_id, branch_id, branch_name)?;
    Ok(SignalBranchObservation::new(
        branch_identity,
        target,
        generation,
    ))
}

impl From<SignalBranchIdentityConstructionDenial> for SignalBranchObservationConstructionDenial {
    fn from(denial: SignalBranchIdentityConstructionDenial) -> Self {
        Self::InvalidIdentity(denial)
    }
}

impl From<SignalBranchTargetConstructionDenial> for SignalBranchObservationConstructionDenial {
    fn from(denial: SignalBranchTargetConstructionDenial) -> Self {
        Self::InvalidTarget(denial)
    }
}
