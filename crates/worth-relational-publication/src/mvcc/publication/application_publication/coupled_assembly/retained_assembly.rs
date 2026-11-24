//! Coupled assembly state that publication carries across its admission boundary.
use super::{
    chain::Chain, completion::CoupledAppliedCandidate, inputs::RequiredOutputInputs,
    producer_posture::RelationalCoupledProducerPosture,
};
use crate::mvcc::publication::application_publication::RelationalAuthoredCoupledOwners;
use crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority;
use crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission;
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

pub struct BoundAuthoredCoupling {
    pub owners: RelationalAuthoredCoupledOwners,
    pub installed: RelationalInstalledApplicationPublicationAuthority,
    pub applied: std::sync::Mutex<Option<CoupledAppliedCandidate>>,
    pub completed: std::sync::Mutex<Option<RequiredOutputPublicationAdmission>>,
    pub retirement_admission: Option<Arc<worth_proof::AdmittedAuthoredOutputRetirement>>,
}
impl std::fmt::Debug for BoundAuthoredCoupling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundAuthoredCoupling")
            .field("owners", &self.owners)
            .finish_non_exhaustive()
    }
}

pub struct CoupledPublicationPart {
    pub inputs: Option<RequiredOutputInputs>,
    pub ordinary_ids:
        Option<worth_execution::ExecutionArray<contracts::identity::data::record_ids::EntityId>>,
    pub input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub publication: RelationalCoupledProducerPosture,
    pub commands: worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub records: Vec<contracts::transactions::data::primitives::RecordRef>,
    /// This route's chain in the candidate: its transaction batches, its patch
    /// fragments, the local-key coordinates its head chose, and the end of the
    /// changed-record log when its latest round applied.
    pub chain: Chain,
}

/// One later authored stage applied over the candidate pin under the authored
/// writer's custody. It is never a producer posture and owns no output route.
pub struct CoupledAuthoredStagePart {
    pub fragments: std::ops::Range<usize>,
    pub roles: Vec<RequiredOutputRoleTarget>,
    pub output_ids: worth_execution::ExecutionArray<contracts::identity::data::record_ids::EntityId>,
    pub source: contracts::identity::data::record_ids::EntityId,
    pub input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub commands: worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub records: Vec<contracts::transactions::data::primitives::RecordRef>,
}
