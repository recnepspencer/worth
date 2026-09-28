use super::{codes, site_of, Fixture};
use crate::diagnostics::DiagnosticCode;
use crate::facade_docs::observation::NameStatus;

const LIB: &str = "pub mod facade;\nmod model;\n";
const FACADE: &str = "pub use crate::model::{Bare, Documented};\n";
const MODEL: &str = "/// A documented value.\npub struct Documented;\n\npub struct Bare;\n";

fn doc_fixture(model: &str) -> Fixture {
    let mut fixture = Fixture::new();
    fixture.crate_with(
        "doc-fixture",
        &[],
        &[("lib.rs", LIB), ("facade.rs", FACADE), ("model.rs", model)],
    );
    fixture
}

#[test]
fn documented_name_passes() {
    let fixture = doc_fixture(MODEL);
    let observation = fixture.observe(&[("doc-fixture", &["Documented"])]);
    assert_eq!(
        observation.status("doc-fixture", "Documented"),
        Some(&NameStatus::Documented)
    );
    assert!(fixture.diagnostics(&observation).is_empty());
}

#[test]
fn undocumented_name_without_debt_fails_with_its_definition_site() {
    let fixture = doc_fixture(MODEL);
    let observation = fixture.observe(&[("doc-fixture", &["Bare", "Documented"])]);
    let diagnostics = fixture.diagnostics(&observation);
    assert_eq!(
        codes(&diagnostics),
        [DiagnosticCode::Bc8004FacadeDocMissing]
    );
    assert_eq!(diagnostics[0].subject(), "doc-fixture::Bare");
    assert!(
        diagnostics[0]
            .message()
            .contains("add a /// doc comment at crates/doc-fixture/src/model.rs:4"),
        "{}",
        diagnostics[0].message()
    );
}

#[test]
fn recorded_debt_admits_the_name_and_paid_debt_goes_stale() {
    let fixture = doc_fixture(MODEL);
    let rows: &[(&str, &[&str])] = &[("doc-fixture", &["Bare", "Documented"])];
    fixture.record_debt(&fixture.observe(rows));
    assert!(fixture.diagnostics(&fixture.observe(rows)).is_empty());

    let paid =
        doc_fixture(&MODEL.replace("pub struct Bare;", "/// Now documented.\npub struct Bare;"));
    paid.record_debt(&fixture.observe(rows));
    let diagnostics = paid.diagnostics(&paid.observe(rows));
    assert_eq!(
        codes(&diagnostics),
        [DiagnosticCode::Bc8005FacadeDocDebtStale]
    );
    assert!(diagnostics[0].message().contains("is now documented"));
}

#[test]
fn debt_for_a_name_no_longer_exported_goes_stale() {
    let fixture = doc_fixture(MODEL);
    fixture.write_debt(
        "schema_version = 1\n\n[[facades]]\npackage = \"doc-fixture\"\nundocumented = [\"Bare\", \"Removed\"]\n",
    );
    let diagnostics = fixture.diagnostics(&fixture.observe(&[("doc-fixture", &["Bare"])]));
    assert_eq!(
        codes(&diagnostics),
        [DiagnosticCode::Bc8005FacadeDocDebtStale]
    );
    assert_eq!(diagnostics[0].subject(), "doc-fixture::Removed");
    assert!(diagnostics[0].message().contains("is no longer exported"));
}

#[test]
fn malformed_debt_fails_closed_and_absent_debt_is_zero_debt() {
    let fixture = doc_fixture(MODEL);
    let observation = fixture.observe(&[("doc-fixture", &["Bare"])]);
    assert_eq!(
        codes(&fixture.diagnostics(&observation)),
        [DiagnosticCode::Bc8004FacadeDocMissing]
    );
    fixture.write_debt(
        "schema_version = 1\n\n[[facades]]\npackage = \"doc-fixture\"\nundocumented = [\"Bare\", \"Bare\"]\n",
    );
    let diagnostics = fixture.diagnostics(&observation);
    assert_eq!(
        codes(&diagnostics),
        [DiagnosticCode::Bc8001SnapshotBaseline]
    );
    assert!(diagnostics[0].message().contains("duplicate value Bare"));
}

#[test]
fn debt_guidance_names_the_definition_and_the_discouraged_regeneration() {
    let fixture = doc_fixture(MODEL);
    let observation = fixture.observe(&[("doc-fixture", &["Bare"])]);
    assert_eq!(
        site_of(&observation, "doc-fixture", "Bare"),
        ["crates/doc-fixture/src/model.rs:4"]
    );
    let rendered = crate::diagnostics::render_human(&fixture.diagnostics(&observation));
    assert!(rendered.contains("crates/doc-fixture/src/model.rs:4; add a `///` doc comment"));
    assert!(rendered.contains("(discouraged)"));
    assert!(rendered.contains("--update-snapshots"));
}
