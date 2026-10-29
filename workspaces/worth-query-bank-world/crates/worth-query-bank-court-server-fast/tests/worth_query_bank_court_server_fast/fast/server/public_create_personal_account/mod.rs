pub(crate) use worth_query_bank_worlds::facade::server::public_create_personal_account::*;

#[test]
fn public_request_creates_reads_and_idempotently_recovers_a_personal_account() {
    assert_public_creation("Everyday account");
    assert_public_creation(&"a".repeat(AccountName::MAX_BYTES));
    assert!(AccountName::new("a".repeat(AccountName::MAX_BYTES + 1)).is_err());
}
