//! Synthetic raw-message parser tests. These do not claim a compiler was run;
//! the consuming courts use real sessions captured by the outer runner.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::denial::{check_outcome, RequiredEnvironment};
use crate::{diagnostics, paths, session, CompilerFixtureRefusal};

const SNAPSHOT: &str = "error[E0603]: function `private` is private\n  --> tests/ui/private.rs:3:2\n   |\n3 | sealed::private();\n   |         ^^^^^^^ private function\n";

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/synthetic/sources/fixture_owner/ui/private.rs")
}

fn synthetic_outcome(source: &Path, environment: &str) -> serde_json::Value {
    let rendered = SNAPSHOT.replace("tests/ui/private.rs", &source.display().to_string());
    json!({
        "environment": environment,
        "fate": "denied",
        "messages": [{
            "level": "error", "message": "function `private` is private",
            "rendered": rendered,
            "spans": [{"file_name": source, "is_primary": true}]
        }]
    })
}

#[test]
fn synthetic_production_capture_preserves_the_complete_snapshot() {
    let source = fixture();
    let outcome = synthetic_outcome(&source, "production");
    let key = paths::source_key(&source);
    let mut raw = json!({"schema":1, "fixtures":{}});
    raw["fixtures"][key.as_str()] = outcome;
    let document = session::parse(&raw.to_string()).unwrap();
    check_outcome(
        &source,
        session::fixture(&document, &source).unwrap(),
        SNAPSHOT,
        RequiredEnvironment::Production,
    )
    .unwrap();
}

#[test]
fn production_denial_refuses_a_certification_capture() {
    let source = fixture();
    let result = check_outcome(
        &source,
        &synthetic_outcome(&source, "certification"),
        SNAPSHOT,
        RequiredEnvironment::Production,
    );
    assert!(matches!(
        result,
        Err(CompilerFixtureRefusal::WrongEnvironment { .. })
    ));
}

#[test]
fn a_successful_compilation_cannot_satisfy_a_denial() {
    let source = fixture();
    let mut outcome = synthetic_outcome(&source, "production");
    outcome["fate"] = json!("compiled");
    assert!(matches!(
        check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Any),
        Err(CompilerFixtureRefusal::UnexpectedCompilerSuccess(_))
    ));
}

#[test]
fn compiler_codes_labels_source_and_coordinates_are_not_normalized_away() {
    let source = fixture();
    for (before, after) in [
        ("E0603", "E0432"),
        ("private function", "unresolved item"),
        ("sealed::private();", "another::private();"),
        (":3:2", ":4:2"),
        (":3:2", ":3:3"),
        ("^^^^^^^", "^^^^^^"),
    ] {
        let mut outcome = synthetic_outcome(&source, "production");
        let rendered = outcome["messages"][0]["rendered"]
            .as_str()
            .unwrap()
            .replace(before, after);
        outcome["messages"][0]["rendered"] = json!(rendered);
        assert!(
            matches!(
                check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Any),
                Err(CompilerFixtureRefusal::WrongDiagnostics { .. })
            ),
            "{before} -> {after}"
        );
    }
}

#[test]
fn same_basename_in_another_file_cannot_supply_attribution() {
    let source = fixture();
    let mut outcome = synthetic_outcome(&source, "production");
    outcome["messages"][0]["spans"][0]["file_name"] =
        json!(source.with_file_name("other").join("private.rs"));
    assert!(matches!(
        check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Any),
        Err(CompilerFixtureRefusal::UnattributedDiagnostics(_))
    ));
}

#[test]
fn missing_rendered_errors_are_refused_instead_of_silently_omitted() {
    let source = fixture();
    for rendered in [serde_json::Value::Null, json!("")] {
        let mut outcome = synthetic_outcome(&source, "production");
        outcome["messages"]
            .as_array_mut()
            .unwrap()
            .push(json!({"level":"error", "spans":[], "rendered": rendered}));
        assert!(matches!(
            check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Any),
            Err(CompilerFixtureRefusal::MissingRenderedDiagnostic { index: 1, .. })
        ));
    }
}

#[test]
fn printed_source_locations_cannot_impersonate_a_raw_fixture_span() {
    let source = fixture();
    let mut outcome = synthetic_outcome(&source, "production");
    outcome["messages"][0]["spans"] = json!([]);
    assert!(matches!(
        check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Any),
        Err(CompilerFixtureRefusal::UnattributedDiagnostics(_))
    ));
}

#[test]
fn a_macro_error_can_be_attributed_through_its_fixture_callsite() {
    let source = fixture();
    let mut outcome = synthetic_outcome(&source, "production");
    outcome["messages"][0]["spans"] = json!([{
        "file_name":"macro_owner.rs", "expansion":{"span":{"file_name":source}}
    }]);
    check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Production).unwrap();
}

#[test]
fn relocated_relative_module_locations_retain_their_full_original_path() {
    let source = fixture();
    let mut outcome = synthetic_outcome(&source, "production");
    let expected = format!("{SNAPSHOT}note: helper\n  ::: tests/ui/support.rs:8:9\n");
    let observed = format!(
        "{}note: helper\n  ::: {}:8:9\n",
        outcome["messages"][0]["rendered"].as_str().unwrap(),
        source.with_file_name("support.rs").display()
    );
    outcome["messages"][0]["rendered"] = json!(observed);
    check_outcome(&source, &outcome, &expected, RequiredEnvironment::Any).unwrap();
    assert_ne!(
        diagnostics::normalize("  --> tests/other/private.rs:3:2", &source),
        diagnostics::normalize("  --> tests/ui/private.rs:3:2", &source)
    );
}

#[test]
fn source_text_containing_an_arrow_is_not_treated_as_a_location_header() {
    let source = fixture();
    let text = format!("3 | // --> {}:3:2", source.display());
    assert_eq!(diagnostics::normalize(&text, &source), text);
}

#[test]
fn absent_raw_errors_or_fixture_entries_are_explicit_refusals() {
    let source = fixture();
    let outcome = json!({"environment":"production", "fate":"denied", "messages":[]});
    assert!(matches!(
        check_outcome(&source, &outcome, "", RequiredEnvironment::Any),
        Err(CompilerFixtureRefusal::NoCompilerErrors(_))
    ));
    let document = session::parse(r#"{"schema":1,"fixtures":{}}"#).unwrap();
    assert!(matches!(
        session::fixture(&document, &source),
        Err(CompilerFixtureRefusal::UnobservedFixture(_))
    ));
}

#[test]
fn unknown_schemas_and_missing_fixture_maps_are_refused() {
    for source in [
        r#"{"schema":2,"fixtures":{}}"#,
        r#"{"schema":1}"#,
        "not JSON",
    ] {
        assert!(matches!(
            session::parse(source),
            Err(CompilerFixtureRefusal::InvalidPrebuild(_))
        ));
    }
}

#[test]
fn recursive_or_character_class_globs_require_explicit_expansion() {
    for path in ["tests/**/private.rs", "tests/[abc].rs", "tests/file?.rs"] {
        assert!(matches!(
            paths::expand(Path::new(path)),
            Err(CompilerFixtureRefusal::UnsupportedPattern(_))
        ));
    }
}

#[test]
fn compiler_warning_text_and_order_remain_part_of_the_snapshot() {
    let source = fixture();
    let mut outcome = synthetic_outcome(&source, "production");
    outcome["messages"].as_array_mut().unwrap().insert(
        0,
        json!({
            "level": "warning", "message": "unreachable expression",
            "rendered": "warning: unreachable expression\n", "spans": []
        }),
    );
    let expected = format!("warning: unreachable expression\n{SNAPSHOT}");
    check_outcome(&source, &outcome, &expected, RequiredEnvironment::Any).unwrap();
    assert!(matches!(
        check_outcome(&source, &outcome, SNAPSHOT, RequiredEnvironment::Any),
        Err(CompilerFixtureRefusal::WrongDiagnostics { .. })
    ));
}
