use worth_foundational::{
    FoundationalAdmittedIdentityValue, FoundationalAuthorityIdentity, FoundationalIdentityKind,
};
use worth_proof::{AuthorityMarker, AuthorityWitness};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

struct QuerySnapshotIdentityKind;
impl FoundationalIdentityKind for QuerySnapshotIdentityKind {}

fn authority() -> AuthorityWitness<QueryIdentityAuthority> {
    AuthorityWitness::from_authority_marker(QueryIdentityAuthority(()))
}

fn needs_commit_authority(
    _identity: FoundationalAuthorityIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind>,
) {
}

fn main() {
    let snapshot = FoundationalAuthorityIdentity::<
        u64,
        QueryIdentityAuthority,
        QuerySnapshotIdentityKind,
    >::from_admitted(FoundationalAdmittedIdentityValue::admit(42, authority()));
    needs_commit_authority(snapshot);
}
