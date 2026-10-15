//! Normal-event guarantees admit the explicit diagnostic dispatch exclusion.
use super::*;

const GUARD: &str = "github.event_name != 'workflow_dispatch' || inputs.scope != 'checkpoint-only'";

fn workflow(guard: &str) -> String {
    format!("on:\n  push:\n  pull_request:\n  workflow_dispatch:\n    inputs:\n      scope:\n        type: choice\n        default: all\n        options: [all, consumer-docs, checkpoint-only]\njobs:\n  docs:\n    if: {guard}\n    steps:\n      - run: python scripts/ci/query_gate.py doc\n")
}

#[test]
fn normal_event_guarantee_preserves_real_doctest_ownership() {
    let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
    for guard in [GUARD.to_owned(), format!("${{{{ {GUARD} }}}}")] {
        fixture.write(".github/workflows/ci.yml", &workflow(&guard));
        assert!(fixture.diagnostics().is_empty());
    }
}

#[test]
fn normal_triggers_require_null_or_configuration_mappings() {
    let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
    for event in ["push", "pull_request"] {
        for declaration in ["false", "true", "42", "'disabled'", "[]"] {
            let ci = workflow(GUARD).replace(
                &format!("  {event}:\n"),
                &format!("  {event}: {declaration}\n"),
            );
            fixture.write(".github/workflows/ci.yml", &ci);
            assert_eq!(
                fixture.codes(),
                [DiagnosticCode::Bc8008ConsumerDocMarker],
                "{ci}"
            );
        }
        for declaration in ["null", "{}", "\n    branches: [master, main]"] {
            let ci = workflow(GUARD).replace(
                &format!("  {event}:\n"),
                &format!("  {event}: {declaration}\n"),
            );
            fixture.write(".github/workflows/ci.yml", &ci);
            assert!(fixture.diagnostics().is_empty(), "{ci}");
        }
    }
}

#[test]
fn unsupported_guards_and_broken_dispatch_contracts_cannot_claim_ownership() {
    let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
    let valid = workflow(GUARD);
    let invalid = [
        workflow("false"),
        workflow("${{ false }}"),
        workflow(&format!("${{{{ {GUARD}")),
        workflow(&GUARD.replace("!= 'workflow_dispatch'", "== 'workflow_dispatch'")),
        workflow(&GUARD.replace("checkpoint-only", "all")),
        valid.replace("  push:\n", ""),
        valid.replace("  pull_request:\n", ""),
        valid.replace("  workflow_dispatch:", "  missing_dispatch:"),
        valid.replace("default: all", "default: checkpoint-only"),
        valid.replace("type: choice", "type: string"),
        valid.replace(
            "[all, consumer-docs, checkpoint-only]",
            "[all, consumer-docs, unknown]",
        ),
        valid.replace(
            "[all, consumer-docs, checkpoint-only]",
            "[all, consumer-docs, checkpoint-only, extra]",
        ),
        valid.replace("    steps:", "    continue-on-error: true\n    steps:"),
        valid.replace("      - run:", "      - if: false\n        run:"),
        valid.replace(
            "      - run:",
            "      - continue-on-error: true\n        run:",
        ),
        valid.replace("query_gate.py doc", "query_gate.py doc-lower"),
    ];
    for ci in invalid {
        fixture.write(".github/workflows/ci.yml", &ci);
        assert_eq!(
            fixture.codes(),
            [DiagnosticCode::Bc8008ConsumerDocMarker],
            "{ci}"
        );
    }
}
