use worth_foundational::{FoundationalIdentityKind, admit_foundational_authority_identity};
use worth_proof::AuthorityMarker;

struct QueryIdentityAuthority(());
impl AuthorityMarker for QueryIdentityAuthority {}

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

fn main() {
    let _identity = admit_foundational_authority_identity::<
        u64,
        QueryIdentityAuthority,
        QueryCommitIdentityKind,
    >(42, ());
}
