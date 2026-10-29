pub(crate) use worth_query_bank_worlds::facade::user_node::protocol_boundary::*;

#[test]
fn user_node_wire_protocol_carries_no_runtime_or_lifecycle_authority() {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../bank-user-node"))
        .join("src/protocol");
    let forbidden = [
        "worth_query",
        "WorthQueryAuthority",
        "BankRequestedEstateElevation",
        "BankApprovedEstateElevation",
        "BankEstateMandatoryReview",
        "BankCommitReceipt",
        "BankCommitRecoveryHandle",
    ];
    let mut hits = Vec::new();
    for path in rust_files(&root) {
        let text = std::fs::read_to_string(&path).expect("read user-node protocol source");
        for needle in forbidden {
            if text.contains(needle) {
                hits.push(format!("{}:{needle}", path.display()));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "user-node wire protocol must remain descriptive and authority-free: {hits:?}"
    );
}
