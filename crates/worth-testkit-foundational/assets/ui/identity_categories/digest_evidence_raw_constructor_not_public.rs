use worth_foundational::{
    CanonicalDerivedDigest, FoundationalDigestIdentityEvidence, FoundationalIdentityBasis,
    FoundationalIdentityKind,
};
use worth_proof::AuthorityMarker;

struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

struct QueryCommitIdentityBasis;
impl FoundationalIdentityBasis for QueryCommitIdentityBasis {}

fn digest() -> CanonicalDerivedDigest {
    loop {}
}

fn main() {
    let _evidence = FoundationalDigestIdentityEvidence::<
        QueryCommitIdentityBasis,
        QueryIdentityAuthority,
        QueryCommitIdentityKind,
    >::from_canonical_digest(digest());
}
