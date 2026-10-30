use worth_foundational::{FoundationalIdentityKind, FoundationalProjectionIdentity};

struct QueryCommitIdentityKind;
impl FoundationalIdentityKind for QueryCommitIdentityKind {}

fn main() {
    let _projection =
        FoundationalProjectionIdentity::<String, QueryCommitIdentityKind>::new("42".to_string());
}
