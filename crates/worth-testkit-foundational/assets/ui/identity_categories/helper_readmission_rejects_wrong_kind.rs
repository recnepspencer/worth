use worth_foundational::{
    FoundationalAdmittedIdentityValue, FoundationalAuthorityIdentity, FoundationalIdentityKind,
    readmit_foundational_authority_identity,
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

fn main() {
    let snapshot = FoundationalAuthorityIdentity::<
        u64,
        QueryIdentityAuthority,
        QuerySnapshotIdentityKind,
    >::from_admitted(FoundationalAdmittedIdentityValue::admit(42, authority()));
    let bridged = snapshot.bridge_trust_boundary();
    let _commit = readmit_foundational_authority_identity::<
        u64,
        QueryIdentityAuthority,
        QueryCommitIdentityKind,
    >(bridged, authority());
}
