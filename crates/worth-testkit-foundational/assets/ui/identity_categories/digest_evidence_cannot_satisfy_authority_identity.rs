use worth_foundational::{
    FoundationalAuthorityIdentity, FoundationalDigestIdentityEvidence, FoundationalIdentityBasis,
    FoundationalIdentityKind,
};
use worth_proof::AuthorityMarker;

struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

struct QueryCommitIdentityBasis;
impl FoundationalIdentityBasis for QueryCommitIdentityBasis {}

fn needs_authority(
    _identity: FoundationalAuthorityIdentity<u64, QueryIdentityAuthority, QueryCommitIdentityKind>,
) {
}

fn digest_evidence() -> FoundationalDigestIdentityEvidence<
    QueryCommitIdentityBasis,
    QueryIdentityAuthority,
    QueryCommitIdentityKind,
> {
    loop {}
}

fn main() {
    needs_authority(digest_evidence());
}
