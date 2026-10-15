use worth_foundational::{
    FoundationalBranchComparisonBasis, FoundationalBranchForkBasis, FoundationalBranchId,
    FoundationalBranchIdConstructionDenial, FoundationalBranchReferenceGeneration,
    FoundationalBranchReferenceObservation, FoundationalBranchTarget,
};

use super::super::target::RelationalBranchTarget;

/// Exact descriptive branch-reference observation, not a repeatable-read artifact.
pub type RelationalBranchReferenceObservation =
    FoundationalBranchReferenceObservation<RelationalBranchTarget>;
pub type RelationalBranchForkBasis = FoundationalBranchForkBasis<RelationalBranchTarget>;
pub type RelationalBranchComparisonBasis =
    FoundationalBranchComparisonBasis<RelationalBranchTarget>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchObservationConstructionDenial {
    InvalidBranchId(FoundationalBranchIdConstructionDenial),
    EmptyBranchName,
    RuntimeInstanceMismatch {
        observation_runtime_instance_id: u64,
        target_runtime_instance_id: u64,
    },
    ForkProvenanceMismatch,
}

/// Lower an owner branch name into the shared exact observation grammar.
pub fn relational_branch_observation(
    runtime_instance_id: u64,
    branch_name: impl AsRef<str>,
    target: FoundationalBranchTarget<RelationalBranchTarget>,
    generation: FoundationalBranchReferenceGeneration,
) -> Result<RelationalBranchReferenceObservation, RelationalBranchObservationConstructionDenial> {
    let branch_name = branch_name.as_ref();
    if branch_name.trim().is_empty() {
        return Err(RelationalBranchObservationConstructionDenial::EmptyBranchName);
    }
    if let FoundationalBranchTarget::Basis(target) = &target
        && target.runtime_instance_id() != runtime_instance_id
    {
        return Err(
            RelationalBranchObservationConstructionDenial::RuntimeInstanceMismatch {
                observation_runtime_instance_id: runtime_instance_id,
                target_runtime_instance_id: target.runtime_instance_id(),
            },
        );
    }
    let branch_id =
        FoundationalBranchId::new(format!("relational/{runtime_instance_id}/{branch_name}"))?;
    Ok(RelationalBranchReferenceObservation::new(
        branch_id, target, generation,
    ))
}

impl From<FoundationalBranchIdConstructionDenial>
    for RelationalBranchObservationConstructionDenial
{
    fn from(denial: FoundationalBranchIdConstructionDenial) -> Self {
        Self::InvalidBranchId(denial)
    }
}
