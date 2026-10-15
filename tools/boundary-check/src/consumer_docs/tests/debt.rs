use super::*;

#[test]
fn new_unclassified_fence_exceeds_debt() {
    let fixture = Fixture::new("```rust\nlet x = 1;\n```\n", 0);
    assert_eq!(
        fixture.codes(),
        [DiagnosticCode::Bc8006ConsumerFenceDebtExceeded]
    );
}

#[test]
fn classified_fence_requires_lowering_paid_debt() {
    let fixture = Fixture::new(
        "<!-- illustrative -->\n```rust,ignore\n// Illustrative: not compiled.\n```\n",
        1,
    );
    assert_eq!(
        fixture.codes(),
        [DiagnosticCode::Bc8007ConsumerFenceDebtStale]
    );
}

#[test]
fn unchanged_debt_and_zero_fence_document_are_valid() {
    assert!(Fixture::new("```rust,no_run\nlet x = 1;\n```\n", 1)
        .diagnostics()
        .is_empty());
    assert!(Fixture::new("# No code\n", 0).diagnostics().is_empty());
}

#[test]
fn debt_rows_must_match_the_exact_inventory() {
    let fixture = Fixture::new("# No code\n", 0);
    fixture.write(DEBT, "documents = []\n");
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8001SnapshotBaseline]);
    fixture.write(DEBT, "[[documents]]\npath = 'docs/guide.md'\nunclassified = 0\n[[documents]]\npath = 'docs/guide.md'\nunclassified = 0\n");
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8001SnapshotBaseline]);
}
