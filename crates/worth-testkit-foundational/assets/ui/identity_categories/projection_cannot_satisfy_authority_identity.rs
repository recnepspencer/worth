use worth_foundational::{
    FoundationalAuthorityIdentity, FoundationalIdentityKind, FoundationalProjectionIdentity,
};
use worth_proof::AuthorityMarker;

struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

fn needs_authority(
    _identity: FoundationalAuthorityIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind>,
) {
}

fn main() {
    let projection = projection_identity();
    needs_authority(projection);
}

fn projection_identity() -> FoundationalProjectionIdentity<String, QueryCommitIdentityKind> {
    loop {}
}
