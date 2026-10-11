use super::{validate_source_identifier_denials, SourceIdentifierDenialConfig};
use crate::diagnostics::{render_human, render_json, DiagnosticCode};
use crate::source_documents::{Counts, Session};
use std::path::Path;

#[test]
fn one_fixture_has_identical_diagnostic_bytes_at_widths_one_two_and_eight() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/source_acquisition");
    let rules = [SourceIdentifierDenialConfig {
        root: "src".into(),
        exclude_paths: Vec::new(),
        forbidden_identifiers: vec!["Declared".into(), "r#type".into(), "alpha".into()],
        forbidden_identifier_fragments: Vec::new(),
        guidance: "fixture refusal".into(),
    }];
    let mut baseline = None;
    for width in [1, 2, 8] {
        let session = Session::new(width);
        let diagnostics = validate_source_identifier_denials(&workspace, &rules);
        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics
            .iter()
            .all(|d| d.code() == DiagnosticCode::Bc2001BandDependencyViolation));
        assert_eq!(
            session.counts(),
            Counts {
                reads: 3,
                parses: 3
            }
        );
        let bytes = (
            render_human(&diagnostics).into_bytes(),
            render_json(&diagnostics).unwrap().into_bytes(),
        );
        match &baseline {
            Some(expected) => assert_eq!(&bytes, expected),
            None => baseline = Some(bytes),
        }
    }
}
