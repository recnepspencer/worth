//! Actual one-error semantic fences; runtime/lease behavior is tested separately.
#[test]
fn owned_publication_custody_is_consumed_and_stopped_authority_cannot_advance() {
    const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/anchored_publication_custody_is_linear.rs");
    let guards: &[(&str, &[&str])] = &[
        ("E0599", &["clone", "AnchoredRecordPublicationOperation"]),
        ("E0599", &["clone", "AnchoredRecordPublicationRecovery"]),
        (
            "E0616",
            &["custody", "AnchoredRecordPublicationOperation", "private"],
        ),
        ("E0599", &["step", "AnchoredRecordStoppedPublication"]),
        ("E0599", &["new", "AnchoredRecordStepTerminal"]),
        (
            "E0382",
            &[
                "AnchoredRecordPublicationOperation",
                "::step",
                "takes ownership of the receiver",
                "operation",
            ],
        ),
        (
            "E0382",
            &[
                "AnchoredRecordStoppedPublication::settle",
                "takes ownership of the receiver",
                "stopped",
            ],
        ),
    ];
    for (code, symbols) in guards {
        worth_testkit_compiler::certification_semantic_denial(SOURCE, code, symbols)
            .expect("actual owner-issued consuming publication boundary");
    }
}
