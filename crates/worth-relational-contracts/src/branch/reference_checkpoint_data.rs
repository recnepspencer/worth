//! Descriptive branch-cell checkpoint and canonical coordinate frame.
use crate::branch::reference::observation::{
    relational_branch_observation, RelationalBranchObservationConstructionDenial,
    RelationalBranchReferenceObservation,
};
use crate::branch::version::RelationalBranchVersion;
use crate::history::data::coordinates::BranchId;
use crate::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;
use worth_foundational::FoundationalBranchTarget;

/// The runtime instance id of the canonical frame. Live runtime ids are issued
/// strictly above it with the same decimal width, so a recorded identity can
/// never be mistaken for a live one and recorded text keeps the live width.
pub const CANONICAL_RUNTIME_INSTANCE_ID: u64 = 1_000_000_000;

/// The transaction id an envelope's merged plan records. Transaction ids are
/// issued per process, one for every transaction begun, reads included, so
/// they are not durable history. Live ids start at one; zero is never issued.
pub const CANONICAL_TRANSACTION_ID: crate::transactions::data::primitives::TransactionId =
    crate::transactions::data::primitives::TransactionId(0);

/// Exact durable image of one owner branch cell. This is intentionally a
/// checkpoint DTO rather than the live cell: restoring it must validate the
/// runtime-affine identity and never synthesize currentness from a legacy
/// branch-head projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RelationalBranchCellCheckpoint {
    pub runtime_instance_id: u64,
    pub branch_id: BranchId,
    pub observation: RelationalBranchReferenceObservation,
    pub truth_version: RelationalBranchVersion,
    pub fork_provenance: Option<RelationalBranchReferenceObservation>,
    pub fork_source_branch_id: Option<BranchId>,
}

impl RelationalBranchCellCheckpoint {
    /// This cell image with every runtime identity in the canonical frame:
    /// the owner, the observed reference and the fork provenance.
    ///
    /// Cost: O(parent count) to rebuild the two targets.
    pub fn in_canonical_frame(
        &self,
    ) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        let observation = canonical_observation(&self.observation, &self.branch_id.0)?;
        let fork_provenance = match (&self.fork_provenance, &self.fork_source_branch_id) {
            (Some(source), Some(source_branch_id)) => {
                Some(canonical_observation(source, &source_branch_id.0)?)
            }
            (None, None) => None,
            _ => return Err(RelationalBranchObservationConstructionDenial::ForkProvenanceMismatch),
        };
        Ok(Self {
            runtime_instance_id: CANONICAL_RUNTIME_INSTANCE_ID,
            branch_id: self.branch_id.clone(),
            observation,
            truth_version: self.truth_version,
            fork_provenance,
            fork_source_branch_id: self.fork_source_branch_id.clone(),
        })
    }
}

pub fn canonical_observation(
    observation: &RelationalBranchReferenceObservation,
    branch_name: &str,
) -> Result<RelationalBranchReferenceObservation, RelationalBranchObservationConstructionDenial> {
    let target = match observation.target() {
        FoundationalBranchTarget::Empty => FoundationalBranchTarget::empty(),
        FoundationalBranchTarget::Basis(target) => FoundationalBranchTarget::basis(
            target.rebind_runtime_instance_id(CANONICAL_RUNTIME_INSTANCE_ID),
        ),
    };
    relational_branch_observation(
        CANONICAL_RUNTIME_INSTANCE_ID,
        branch_name,
        target,
        observation.generation(),
    )
}
