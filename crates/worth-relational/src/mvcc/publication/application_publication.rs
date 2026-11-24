mod publication_appendix;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
pub use publication_appendix::{
    RelationalPublicationAdoptionRequest, RelationalPublicationAppendix,
    RelationalPublicationBodySelection,
};
pub(crate) mod historical_selection;
pub(crate) use historical_selection::require_graph_coverage;
pub use historical_selection::RelationalHistoricalOutputSelection;
mod authored_cascade;
mod authored_publication_reads;
pub(crate) use authored_publication_reads::AuthoredPublicationReads;
mod authored_restoration;
mod authored_retirement;
pub use authored_retirement::AuthoredOwnedMemberKind;
pub(crate) mod native_transition_succession;
pub(crate) mod restored_producer_prior;
pub use authored_cascade::RelationalAuthoredRetirement;
pub use authored_restoration::RelationalAuthoredRestoration;
pub use restored_producer_prior::RelationalRestoredProducerPrior;
mod completion;
pub(crate) mod coupled_assembly;
mod coupled_owners;
pub(crate) mod custody;
pub(crate) use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning;
mod ownership_reach;
pub use coupled_assembly::assembly_failure::RelationalCoupledAssemblyFailure;
pub use coupled_assembly::completion::RelationalCoupledAuthoredPublication;
pub use coupled_assembly::contribution::RelationalCoupledContribution;
pub use coupled_assembly::inputs::RelationalRequiredOutputInput;
pub use coupled_assembly::producer_posture::RelationalCoupledProducerPosture;
pub use coupled_assembly::source_effects::RelationalCoupledAuthoredSourceEffects;
pub(crate) use coupled_assembly::{
    inputs::RequiredOutputInputs, retained_assembly::BoundAuthoredCoupling,
};
pub(crate) use coupled_owners::CoupledSource;
pub use coupled_owners::{RelationalAuthoredCoupledOwners, RelationalDeclaredCoupledOwner};
pub(crate) mod predecessor_basis;
mod predecessor_observation;
mod proposal_binding;
mod proposal_predecessor;
pub use predecessor_basis::{
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin, RelationalPredecessorInput,
};
pub(crate) use worth_relational_contracts::internal_port::mvcc::publication::application_publication::producing_implementation;
pub(crate) mod route_absence;
mod scope_ownership;
pub use completion::RequiredOutputPublication;
pub(crate) use custody::prepare as prepare_publications;
pub use custody::{
    RecordedRequiredOutputRole, RelationalPerformedOutputPublications,
    RelationalPreparedOutputPublications, RelationalRequiredOutputPublication,
};
pub use output_meaning::{
    RequiredOutputPublicationMeaning, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
pub use producing_implementation::RequiredOutputProducingImplementation;
pub use route_absence::RelationalRequiredOutputRouteAbsence;

#[cfg(test)]
mod absent_optional_field;
#[cfg(test)]
mod continuity_tests;
#[cfg(test)]
mod tests;

/// Native installation binding for one exact application publisher occurrence.
/// Descriptive producer names cannot construct or substitute this binding.
#[derive(Clone, Debug)]
pub struct RelationalInstalledApplicationPublicationAuthority {
    pub(crate) runtime_instance_id: u64,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) schema_digest: [u8; 32],
    pub(crate) source_scope_ownership:
        crate::schema::installed_source_scopes::InstalledSourceScopeOwnership,
    pub(crate) authority: worth_proof::InstalledApplicationPublicationAuthority,
}

#[derive(Debug)]
pub(crate) struct RequiredOutputPublicationAdmission {
    pub(crate) inputs: RequiredOutputInputs,
    pub(crate) installed: RelationalInstalledApplicationPublicationAuthority,
    pub(crate) publication: RequiredOutputPublication,
}

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn admits_checkpoint_program_activation(
        &self,
        proof: &worth_proof::AdmittedCheckpointProgramActivation,
    ) -> bool {
        self.authority.admits_checkpoint_program_activation(proof)
    }
}

pub(crate) use crate::transactions::data::outcomes::transaction_commit_error::origin_denial;

#[cfg(test)]
pub(crate) fn native_birth_fixture(
    admission: worth_proof::AdmittedRequiredOutputPublication,
    targets: Vec<crate::transactions::data::primitives::CreatedEntityRef>,
) -> RequiredOutputPublication {
    // Closed native fixtures declare a distinct family for each independent
    // producer; continuation fixtures retain the original family's meaning.
    let family = admission.producer().to_owned();
    RequiredOutputPublication::Birth {
        admission,
        absence: None,
        meaning: meaning_fixture::native_meaning_fixture(&family),
        roles: targets
            .into_iter()
            .enumerate()
            .map(|(index, target)| RequiredOutputRoleTarget {
                role: format!("output-{index}"),
                entity_name: "native-unit-output".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: crate::transactions::data::primitives::EntityReference::Created(target),
            })
            .collect(),
    }
}

#[cfg(test)]
#[path = "application_publication_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub(crate) use publication_appendix::prepare_publication_appendix;

#[cfg(test)]
pub(crate) mod meaning_fixture;
