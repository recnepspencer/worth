//! The exact coupled publication and its applied candidate custody.
use super::{
    retained_assembly::{BoundAuthoredCoupling, CoupledAuthoredStagePart, CoupledPublicationPart},
    source_effects::RelationalCoupledAuthoredSourceEffects,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationCore;
use crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use std::sync::Arc;

/// The Native-applied group retains each writer's concrete admission and commands.
/// Its constructor is private; declarations and submitted rosters cannot mint it.
pub struct RelationalCoupledAuthoredPublication {
    pub(crate) binding: Arc<BoundAuthoredCoupling>,
    pub(crate) source_effects: RelationalCoupledAuthoredSourceEffects,
    pub(crate) authored_records: Vec<crate::transactions::data::primitives::RecordRef>,
    // This prefix of the same retained vector is fixed before any other writer.
    pub(crate) authored_prefix_records: usize,
    pub(crate) authored_commands:
        Arc<crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan>,
    pub(crate) parts: Vec<CoupledPublicationPart>,
    pub(crate) stages: Vec<CoupledAuthoredStagePart>,
    pub(super) _authored_records_memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl std::fmt::Debug for RelationalCoupledAuthoredPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalCoupledAuthoredPublication")
            .field("owners", &self.binding.owners)
            .field("parts", &self.parts.len())
            .finish_non_exhaustive()
    }
}
impl RelationalCoupledAuthoredPublication {
    pub(crate) fn admitted_by(
        &self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
    ) -> bool {
        self.binding.installed.runtime_instance_id == installed.runtime_instance_id
            && self
                .parts
                .iter()
                .all(|part| part.publication.admitted_by(installed))
            && self.binding.retirement_admission.as_ref().map_or(
                self.binding.owners.retirements.is_empty(),
                |admission| {
                    self.binding.owners.retirements.iter().all(|prior| {
                        installed
                            .authority
                            .admits_authored_output_retirement(admission, prior.authority())
                    })
                },
            )
    }
}

/// The applied image is held only until the existing ordinary apply phase
/// adopts it. It is neither a preview nor a second application of its commands.
pub(crate) struct CoupledAppliedCandidate {
    pub(crate) identity: RelationalMutationProposalIdentity,
    pub(crate) commands: crate::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub(crate) mutation: AppliedMutationCore,
}
