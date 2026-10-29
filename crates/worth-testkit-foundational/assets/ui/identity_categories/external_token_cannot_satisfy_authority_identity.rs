use worth_foundational::{
    FoundationalAuthorityIdentity, FoundationalExternalIdentityToken, FoundationalIdentityKind,
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
    let token = FoundationalExternalIdentityToken::<u64, QueryCommitIdentityKind>::new(42);
    needs_authority(token);
}
