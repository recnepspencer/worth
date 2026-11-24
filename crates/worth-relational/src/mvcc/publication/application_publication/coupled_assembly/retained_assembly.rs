//! Coupled assembly state that publication carries across its admission boundary.
use super::{
    chain::Chain, completion::CoupledAppliedCandidate, inputs::RequiredOutputInputs,
    producer_posture::RelationalCoupledProducerPosture,
};
use crate::mvcc::publication::application_publication::{
    RelationalAuthoredCoupledOwners, RelationalInstalledApplicationPublicationAuthority,
    RequiredOutputPublicationAdmission, RequiredOutputRoleTarget,
};
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use std::sync::Arc;

pub(crate) struct BoundAuthoredCoupling {
    pub(crate) owners: RelationalAuthoredCoupledOwners,
    pub(crate) installed: RelationalInstalledApplicationPublicationAuthority,
    pub(crate) applied: std::sync::Mutex<Option<CoupledAppliedCandidate>>,
    pub(crate) completed: std::sync::Mutex<Option<RequiredOutputPublicationAdmission>>,
    pub(crate) retirement_admission: Option<Arc<worth_proof::AdmittedAuthoredOutputRetirement>>,
}
impl std::fmt::Debug for BoundAuthoredCoupling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundAuthoredCoupling")
            .field("owners", &self.owners)
            .finish_non_exhaustive()
    }
}

pub(crate) struct CoupledPublicationPart {
    pub(crate) inputs: Option<RequiredOutputInputs>,
    pub(crate) ordinary_ids:
        Option<worth_execution::ExecutionArray<crate::identity::data::EntityId>>,
    pub(crate) input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub(crate) publication: RelationalCoupledProducerPosture,
    pub(crate) commands: crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub(crate) records: Vec<crate::transactions::data::primitives::RecordRef>,
    /// This route's chain in the candidate: its transaction batches, its patch
    /// fragments, the local-key coordinates its head chose, and the end of the
    /// changed-record log when its latest round applied.
    pub(crate) chain: Chain,
    /// The naming plan of the chain's latest round, rescoped with its
    /// commands; a successor round's plan replaces its predecessor's, whose
    /// batches are withdrawn. Taken once into the envelope's plan.
    pub(crate) naming: Option<crate::lineage::data::CommitNamingPlan>,
}

/// One later authored stage applied over the candidate pin under the authored
/// writer's custody. It is never a producer posture and owns no output route.
pub(crate) struct CoupledAuthoredStagePart {
    pub(crate) fragments: std::ops::Range<usize>,
    pub(crate) roles: Vec<RequiredOutputRoleTarget>,
    pub(crate) output_ids: worth_execution::ExecutionArray<crate::identity::data::EntityId>,
    pub(crate) source: crate::identity::data::EntityId,
    pub(crate) input: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    pub(crate) commands: crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub(crate) records: Vec<crate::transactions::data::primitives::RecordRef>,
    /// The stage's naming plan, rescoped with its commands.
    pub(crate) naming: Option<crate::lineage::data::CommitNamingPlan>,
}
