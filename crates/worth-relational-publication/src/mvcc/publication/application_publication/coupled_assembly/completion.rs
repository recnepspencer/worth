//! The exact coupled publication and its applied candidate custody.
use super::{
    retained_assembly::{BoundAuthoredCoupling, CoupledAuthoredStagePart, CoupledPublicationPart},
    source_effects::RelationalCoupledAuthoredSourceEffects,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationCore;
use crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority;
use mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

/// The Native-applied group retains each writer's concrete admission and commands.
/// Its constructor is private; declarations and submitted rosters cannot mint it.
pub struct RelationalCoupledAuthoredPublication {
    pub(crate) binding: Arc<BoundAuthoredCoupling>,
    pub(crate) source_effects: RelationalCoupledAuthoredSourceEffects,
    pub(crate) authored_records: Vec<contracts::transactions::data::primitives::RecordRef>,
    // This prefix of the same retained vector is fixed before any other writer.
    pub(crate) authored_prefix_records: usize,
    pub(crate) authored_commands:
        Arc<mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan>,
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
pub struct CoupledAppliedCandidate {
    pub identity: RelationalMutationProposalIdentity,
    pub commands: mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan,
    pub mutation: AppliedMutationCore,
}

#[doc(hidden)]
pub trait RelationalCoupledAuthoredPublicationFieldsPort: Sized {
    fn authored_commands(
        &self,
    ) -> &Arc<mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan>;
    fn authored_prefix_records(&self) -> usize;
    fn authored_records(&self) -> &Vec<contracts::transactions::data::primitives::RecordRef>;
    fn binding(&self) -> &Arc<BoundAuthoredCoupling>;
    fn parts(&self) -> &Vec<CoupledPublicationPart>;
    fn stages(&self) -> &Vec<CoupledAuthoredStagePart>;
}

impl RelationalCoupledAuthoredPublicationFieldsPort for RelationalCoupledAuthoredPublication {
    fn authored_commands(
        &self,
    ) -> &Arc<mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan> {
        &self.authored_commands
    }
    fn authored_prefix_records(&self) -> usize {
        self.authored_prefix_records
    }
    fn authored_records(&self) -> &Vec<contracts::transactions::data::primitives::RecordRef> {
        &self.authored_records
    }
    fn binding(&self) -> &Arc<BoundAuthoredCoupling> {
        &self.binding
    }
    fn parts(&self) -> &Vec<CoupledPublicationPart> {
        &self.parts
    }
    fn stages(&self) -> &Vec<CoupledAuthoredStagePart> {
        &self.stages
    }
}

#[doc(hidden)]
pub trait RelationalCoupledAuthoredPublicationOwnerPort: Sized {
    fn from_completed_group(
        binding: Arc<BoundAuthoredCoupling>,
        source_effects: RelationalCoupledAuthoredSourceEffects,
        authored_records: Vec<contracts::transactions::data::primitives::RecordRef>,
        authored_prefix_records: usize,
        authored_commands: Arc<mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan>,
        parts: Vec<CoupledPublicationPart>,
        stages: Vec<CoupledAuthoredStagePart>,
        _authored_records_memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self;
}

impl RelationalCoupledAuthoredPublicationOwnerPort for RelationalCoupledAuthoredPublication {
    fn from_completed_group(
        binding: Arc<BoundAuthoredCoupling>,
        source_effects: RelationalCoupledAuthoredSourceEffects,
        authored_records: Vec<contracts::transactions::data::primitives::RecordRef>,
        authored_prefix_records: usize,
        authored_commands: Arc<mutation::mvcc::transaction::normalized_plan::OwnedNormalizedPlan>,
        parts: Vec<CoupledPublicationPart>,
        stages: Vec<CoupledAuthoredStagePart>,
        _authored_records_memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            binding,
            source_effects,
            authored_records,
            authored_prefix_records,
            authored_commands,
            parts,
            stages,
            _authored_records_memory,
        }
    }
}
