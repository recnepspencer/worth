use worth_foundational::{
    FoundationalAdmittedIdentityValue, FoundationalAuthorityIdentity, FoundationalIdentityKind,
    project_foundational_identity,
};
use worth_proof::{AuthorityMarker, AuthorityWitness};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

fn authority() -> AuthorityWitness<QueryIdentityAuthority> {
    AuthorityWitness::from_authority_marker(QueryIdentityAuthority(()))
}

fn main() {
    let identity = FoundationalAuthorityIdentity::<
        u64,
        QueryIdentityAuthority,
        QueryCommitIdentityKind,
    >::from_admitted(FoundationalAdmittedIdentityValue::admit(42, authority()));
    let _projection = project_foundational_identity(&identity, "query:commit:42".to_string(), ());
}
