use worth_foundational::{
    derive_canonical_digest, FoundationalProfileIdentity,
};

fn requires_profile_identity(_: FoundationalProfileIdentity) {}

fn main() {
    let digest = derive_canonical_digest(panic!("type-check only"));
    requires_profile_identity(digest);
}
