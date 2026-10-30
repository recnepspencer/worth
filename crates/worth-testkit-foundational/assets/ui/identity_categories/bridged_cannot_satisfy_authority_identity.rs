use worth_foundational::{
    FoundationalAuthorityIdentity, FoundationalBoundaryBridgedIdentity, FoundationalIdentityKind,
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

fn bridged_identity(
) -> FoundationalBoundaryBridgedIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind> {
    loop {}
}

fn main() {
    needs_authority(bridged_identity());
}
