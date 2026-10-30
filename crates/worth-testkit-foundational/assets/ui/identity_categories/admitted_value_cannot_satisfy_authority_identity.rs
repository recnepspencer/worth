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

fn needs_authority(
    _identity: FoundationalAuthorityIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind>,
) {
}

fn main() {
    let admitted = FoundationalAdmittedIdentityValue::<
        u64,
        QueryIdentityAuthority,
        QueryCommitIdentityKind,
    >::admit(42, authority());
    needs_authority(admitted);
}
