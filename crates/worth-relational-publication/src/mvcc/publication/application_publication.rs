use worth_relational_contracts::internal_port as contracts;
pub mod publication_appendix;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
pub use publication_appendix::{
    RelationalPublicationAdoptionRequest, RelationalPublicationAppendix,
    RelationalPublicationBodySelection,
};
pub mod historical_selection;
pub use historical_selection::RelationalHistoricalOutputSelection;
pub mod authored_cascade;
pub mod authored_publication_reads;
pub(crate) use authored_publication_reads::AuthoredPublicationReads;
pub mod authored_restoration;
pub mod authored_retirement;
pub use authored_retirement::AuthoredOwnedMemberKind;
pub mod native_transition_succession;
pub mod restored_producer_prior;
pub use authored_cascade::RelationalAuthoredRetirement;
pub use authored_restoration::RelationalAuthoredRestoration;
pub use restored_producer_prior::RelationalRestoredProducerPrior;
pub mod completion;
pub mod coupled_assembly;
pub mod coupled_owners;
pub mod custody;
pub mod ownership_reach;
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
pub mod predecessor_basis;
pub mod predecessor_observation;

pub use predecessor_basis::{
    RelationalPredecessorBasisDenial, RelationalPredecessorBasisOrigin, RelationalPredecessorInput,
};
pub mod route_absence;
pub mod scope_ownership;
pub use completion::RequiredOutputPublication;
pub use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
pub use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture;
pub use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
pub use contracts::mvcc::publication::application_publication::producing_implementation::RequiredOutputProducingImplementation;
pub(crate) use custody::prepare as prepare_publications;
pub use custody::{
    RecordedRequiredOutputRole, RelationalPerformedOutputPublications,
    RelationalPreparedOutputPublications, RelationalRequiredOutputPublication,
};
pub use route_absence::RelationalRequiredOutputRouteAbsence;

#[doc(hidden)]
pub trait RelationalInstalledApplicationPublicationAuthorityOwnerPort: Sized {
    fn from_installation(
        runtime_instance_id: u64,
        custom_invariant_generation: u64,
        schema_digest: [u8; 32],
        authority: worth_proof::InstalledApplicationPublicationAuthority,
        source_scope_ownership: contracts::schema::installed_source_scopes::InstalledSourceScopeOwnership,
    ) -> Self;

    fn admits_checkpoint_program_activation(
        &self,
        proof: &worth_proof::AdmittedCheckpointProgramActivation,
    ) -> bool;
}

impl RelationalInstalledApplicationPublicationAuthorityOwnerPort
    for RelationalInstalledApplicationPublicationAuthority
{
    fn from_installation(
        runtime_instance_id: u64,
        custom_invariant_generation: u64,
        schema_digest: [u8; 32],
        authority: worth_proof::InstalledApplicationPublicationAuthority,
        source_scope_ownership: contracts::schema::installed_source_scopes::InstalledSourceScopeOwnership,
    ) -> Self {
        Self {
            runtime_instance_id,
            custom_invariant_generation,
            schema_digest,
            authority,
            source_scope_ownership,
        }
    }

    fn admits_checkpoint_program_activation(
        &self,
        proof: &worth_proof::AdmittedCheckpointProgramActivation,
    ) -> bool {
        self.admits_checkpoint_program_activation(proof)
    }
}

#[doc(hidden)]
pub trait RelationalInstalledApplicationPublicationAuthorityFieldsPort: Sized {
    fn authority(&self) -> &worth_proof::InstalledApplicationPublicationAuthority;
    fn custom_invariant_generation(&self) -> u64;
    fn runtime_instance_id(&self) -> u64;
    fn schema_digest(&self) -> [u8; 32];
    fn source_scope_ownership(
        &self,
    ) -> &contracts::schema::installed_source_scopes::InstalledSourceScopeOwnership;
}

impl RelationalInstalledApplicationPublicationAuthorityFieldsPort
    for RelationalInstalledApplicationPublicationAuthority
{
    fn authority(&self) -> &worth_proof::InstalledApplicationPublicationAuthority {
        &self.authority
    }
    fn custom_invariant_generation(&self) -> u64 {
        self.custom_invariant_generation
    }
    fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
    fn schema_digest(&self) -> [u8; 32] {
        self.schema_digest
    }
    fn source_scope_ownership(
        &self,
    ) -> &contracts::schema::installed_source_scopes::InstalledSourceScopeOwnership {
        &self.source_scope_ownership
    }
}

#[doc(hidden)]
#[path = "application_publication/continuity_tests/payload_forms/schema.rs"]
pub mod payload_forms_schema;

/// Native installation binding for one exact application publisher occurrence.
/// Descriptive producer names cannot construct or substitute this binding.
#[derive(Clone, Debug)]
pub struct RelationalInstalledApplicationPublicationAuthority {
    pub(crate) runtime_instance_id: u64,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) schema_digest: [u8; 32],
    pub(crate) source_scope_ownership:
        contracts::schema::installed_source_scopes::InstalledSourceScopeOwnership,
    pub(crate) authority: worth_proof::InstalledApplicationPublicationAuthority,
}

#[derive(Debug)]
pub struct RequiredOutputPublicationAdmission {
    pub inputs: RequiredOutputInputs,
    pub installed: RelationalInstalledApplicationPublicationAuthority,
    pub publication: RequiredOutputPublication,
}

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn admits_checkpoint_program_activation(
        &self,
        proof: &worth_proof::AdmittedCheckpointProgramActivation,
    ) -> bool {
        self.authority.admits_checkpoint_program_activation(proof)
    }
}

pub(crate) use contracts::transactions::data::outcomes::transaction_commit_error::origin_denial;

#[doc(hidden)]
pub fn native_birth_for_test(
    admission: worth_proof::AdmittedRequiredOutputPublication,
    targets: Vec<contracts::transactions::data::primitives::CreatedEntityRef>,
) -> RequiredOutputPublication {
    // Closed native fixtures declare a distinct family for each independent
    // producer; continuation fixtures retain the original family's meaning.
    let family = admission.producer().to_owned();
    RequiredOutputPublication::Birth {
        admission,
        absence: None,
        meaning: meaning_fixture::native_meaning_for_test(&family),
        roles: targets
            .into_iter()
            .enumerate()
            .map(|(index, target)| RequiredOutputRoleTarget {
                role: format!("output-{index}"),
                entity_name: "native-unit-output".into(),
                kind: target.kind_id,
                posture: RequiredOutputRolePosture::Create,
                target: contracts::transactions::data::primitives::EntityReference::Created(target),
            })
            .collect(),
    }
}

#[cfg(test)]
#[path = "application_publication_inline_type_budget.rs"]
pub mod inline_type_budget;

use worth_relational_branch::internal_port::mvcc::publication::application_publication::meaning_fixture;
