//! One record model owns field lowering and reusable scalar bindings.
use super::super::attempts::data::*;
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct ProposalRecords;
impl fragment::Fragment for ProposalRecords {
    const NAME: &'static str = "proposal_records";
    type Bindings = ProposalRecordsBindings;
    type Members = (
        (
            ExternalMapping,
            Principal,
            Registry,
            Model,
            Target,
            Proposal,
            MutationGrant,
            AcceptanceClaim,
        ),
        (AcceptanceResult, Grant),
    );
}
pub struct ProposalRecordsBindings;
impl fragment::BindingSet for ProposalRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for ProposalRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for ProposalRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for ProposalRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for ProposalRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<StoredProposal> for ProposalRecordsBindings {
    type Binding = StoredProposalBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<StoredClaim> for ProposalRecordsBindings {
    type Binding = ClaimDataBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<StoredResult> for ProposalRecordsBindings {
    type Binding = ResultDataBinding;

    const WRITABLE: bool = false;
}

#[path = "records/entity_members.rs"]
mod entity_members;
pub use entity_members::{
    AcceptanceClaim, AcceptanceResult, ExternalMapping, Grant, Model, MutationGrant, Principal,
    Proposal, Registry, Target,
};
#[path = "records/value_rows.rs"]
mod value_rows;
pub use value_rows::{
    ClaimFacts, ExternalIdentity, GrantFacts, ModelFacts, MutationGrantFacts, PrincipalFacts,
    ProposalFacts, RegistryFacts, ResultFacts, TargetFacts,
};
#[path = "records/selections.rs"]
mod selections;
pub use selections::{
    ClaimData, ClaimName, ExternalIdentityField, GrantAction, GrantDepth, GrantDisclosure,
    GrantIdentity, GrantNotAfter, GrantNotBefore, GrantPurpose, GrantReadKind, GrantReadStatus,
    GrantScope, GrantStatus, MappingStatus, ModelName, MutationGrantAction, MutationGrantDepth,
    MutationGrantIdentity, MutationGrantNotAfter, MutationGrantNotBefore, MutationGrantPurpose,
    MutationGrantScope, MutationGrantStatus, PrincipalIdentity, ProposalData, ProposalName,
    RegistryName, ResultData, ResultName, Retention, TargetName,
};
#[path = "records/relations.rs"]
mod relations;
pub use relations::{
    GrantGrantee, GrantGrantor, GrantParent, GrantResource, MappingTarget, ModelClaim,
    ModelProposal, ModelResult, ModelTarget, MutationGrantGrantee, MutationGrantGrantor,
    MutationGrantParent, MutationGrantResource,
};
