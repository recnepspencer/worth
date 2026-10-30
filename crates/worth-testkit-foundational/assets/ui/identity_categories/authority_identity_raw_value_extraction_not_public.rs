use worth_foundational::{
    FoundationalAdmittedIdentityValue, FoundationalAuthorityIdentity, FoundationalIdentityKind,
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
    let _raw_value = identity.into_value();
}
