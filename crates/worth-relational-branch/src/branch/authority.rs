use worth_proof::AuthorityWitness;

use super::fork_source_basis::AdmittedRelationalForkSourceBasis;
worth_proof::authority_marker!(pub RelationalBranchObservationAuthorityMarker);
worth_proof::authority_marker!(pub RelationalBranchMutationAuthorityMarker);
worth_proof::authority_marker!(pub RelationalBranchPublicationAuthorityMarker);
worth_proof::authority_marker!(pub RelationalForkSourceAuthorityMarker);

pub type RelationalForkSourceAuthority = AuthorityWitness<RelationalForkSourceAuthorityMarker>;
pub type RelationalBranchObservationAuthority =
    AuthorityWitness<RelationalBranchObservationAuthorityMarker>;
pub type RelationalBranchMutationAuthority =
    AuthorityWitness<RelationalBranchMutationAuthorityMarker>;
pub type RelationalBranchPublicationAuthority =
    AuthorityWitness<RelationalBranchPublicationAuthorityMarker>;

pub(super) fn issue_relational_branch_observation_authority() -> RelationalBranchObservationAuthority
{
    RelationalBranchObservationAuthorityMarker::witness()
}

pub fn issue_relational_branch_mutation_authority() -> RelationalBranchMutationAuthority {
    RelationalBranchMutationAuthorityMarker::witness()
}

pub fn issue_relational_branch_publication_authority() -> RelationalBranchPublicationAuthority {
    RelationalBranchPublicationAuthorityMarker::witness()
}

pub fn admit_relational_fork_source(
    descriptor: super::fork_source_basis::RelationalForkSourceDescriptor,
) -> AdmittedRelationalForkSourceBasis {
    AdmittedRelationalForkSourceBasis::new(
        descriptor,
        RelationalForkSourceAuthorityMarker::witness(),
    )
}
