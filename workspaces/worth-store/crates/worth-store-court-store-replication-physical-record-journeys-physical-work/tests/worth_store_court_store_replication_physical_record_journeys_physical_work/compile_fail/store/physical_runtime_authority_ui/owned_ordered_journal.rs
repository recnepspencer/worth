//! Each intended guard must join its own actual code, API and fixture attribution.
#[test]
fn owned_journal_endpoints_and_actual_receipt_custody_cannot_be_reused_or_repaired_by_callers() {
    const SOURCE:&str=concat!(env!("CARGO_MANIFEST_DIR"),"/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/ordered_journal_publication_custody_is_linear.rs");
    let guards: &[(&str, &[&str])] = &[
        ("E0599", &["clone", "OrderedJournalPublicationOperation"]),
        ("E0599", &["clone", "OrderedJournalPublicationRecovery"]),
        ("E0599", &["step", "OrderedJournalStoppedPublication"]),
        ("E0616", &["terminal", "OrderedJournalCommitted", "private"]),
        (
            "E0382",
            &[
                "OrderedJournalPublicationOperation",
                "::step",
                "takes ownership of the receiver",
                "operation",
            ],
        ),
        (
            "E0382",
            &[
                "OrderedJournalStoppedPublication::settle",
                "takes ownership of the receiver",
                "stopped",
            ],
        ),
        (
            "E0382",
            &[
                "OrderedJournalPublicationRecovery::resume",
                "takes ownership of the receiver",
                "recovery",
            ],
        ),
    ];
    for (code, symbols) in guards {
        worth_testkit_compiler::certification_semantic_denial(SOURCE, code, symbols)
            .expect("actual owned journal endpoint/receipt boundary");
    }
}
