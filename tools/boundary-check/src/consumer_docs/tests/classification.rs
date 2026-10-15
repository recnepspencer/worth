use super::*;

#[test]
fn valid_compiled_doctest_and_visible_illustration_are_classified() {
    let fixture = Fixture::new("<!-- compiled: example.rs -->\n```rust\nlet x = 1;\n```\n\n<!-- doctest: example -->\n```rust,no_run\nlet x = 2;\n```\n\n<!-- illustrative -->\n```rust,ignore\n// Illustrative: not compiled.\nlet x = 3;\n```\n", 0);
    fixture.source("// compiled-example: begin\nfn example() {}\n// compiled-example: end\n");
    assert!(
        fixture.diagnostics().is_empty(),
        "{}",
        crate::diagnostics::render_human(&fixture.diagnostics())
    );
}

#[test]
fn invalid_marker_family_fails_bc8008() {
    let cases = [
        "<!-- compiled: example.rs -->\n```rust,ignore\nlet x = 1;\n```\n",
        "<!-- compiled: absent.rs -->\n```rust\nlet x = 1;\n```\n",
        "<!-- compiled: ../example.rs -->\n```rust\nlet x = 1;\n```\n",
        "<!-- doctest: absent -->\n```rust\nlet x = 1;\n```\n",
        "<!-- illustrative -->\n```rust\n// Illustrative: not compiled.\n```\n",
        "<!-- illustrative -->\n```rust,ignore\nlet x = 1;\n```\n",
        "<!-- compiled: -->\n```rust\nlet x = 1;\n```\n",
        "<!-- compiled: example.rs -->\n\n```rust\nlet x = 1;\n```\n",
        "<!-- illustrative -->\ntext\n",
        "<!-- illustrative -->\n```text\n// Illustrative: not compiled.\n```\n",
    ];
    for text in cases {
        let fixture = Fixture::new(text, 0);
        fixture.source("// compiled-example: begin\nfn example() {}\n// compiled-example: end\n");
        assert!(
            fixture
                .codes()
                .contains(&DiagnosticCode::Bc8008ConsumerDocMarker),
            "{text}"
        );
    }
}

#[test]
fn missing_duplicate_and_reversed_source_regions_are_invalid() {
    for source in [
        "fn example() {}\n",
        "// compiled-example: end\n// compiled-example: begin\n",
        "// compiled-example: begin\n// compiled-example: begin\n// compiled-example: end\n",
    ] {
        let fixture = Fixture::new(
            "<!-- compiled: example.rs -->\n```rust\nlet x = 1;\n```\n",
            0,
        );
        fixture.source(source);
        assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    }
}

#[test]
fn inclusion_and_cargo_doctest_selection_cannot_be_counterfeited() {
    let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
    fixture.write(
        "query/crates/example/src/lib.rs",
        "// include_str!(\"../../../../docs/guide.md\")\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write(
        "query/crates/example/src/lib.rs",
        "#![doc = include_str!(\"../../../../docs/guide.md\")]\n",
    );
    fixture.write(
        "query/crates/example/Cargo.toml",
        "[package]\nname = 'example'\nversion = '0.1.0'\n[lib]\ndoctest = false\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
}

#[test]
fn a_detached_source_file_cannot_claim_compilation() {
    let fixture = Fixture::new(
        "<!-- compiled: example.rs -->\n```rust\nlet x = 1;\n```\n",
        0,
    );
    fixture.source("// compiled-example: begin\nfn example() {}\n// compiled-example: end\n");
    fixture.write(&format!("{SOURCE}/lib.rs"), "// no module declaration\n");
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
}

#[test]
fn markers_inside_code_are_example_contents_and_crlf_is_valid() {
    let fixture = Fixture::new("```text\n<!-- illustrative -->\n```\n\n<!-- illustrative -->\r\n```rust,ignore\r\n// Illustrative: not compiled.\r\n```\r\n", 0);
    assert!(
        fixture.diagnostics().is_empty(),
        "{}",
        crate::diagnostics::render_human(&fixture.diagnostics())
    );
}

#[test]
fn doctest_requires_a_live_ci_lane_and_unconditional_inclusion() {
    let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
    fixture.write(
        ".github/workflows/ci.yml",
        "# run: python scripts/ci/query_gate.py doc\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write(".github/workflows/ci.yml", "jobs:\n  docs:\n    steps:\n      - name: docs\n        run: python scripts/ci/query_gate.py doc\n");
    fixture.write(
        "query/crates/example/src/lib.rs",
        "#![cfg(test)]\n#![doc = include_str!(\"../../../../docs/guide.md\")]\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
}

#[test]
fn nested_fences_are_counted_and_can_be_visibly_classified() {
    let fixture = Fixture::new("- An excerpt:\n\n  ```rust\n  let x = 1;\n  ```\n", 0);
    assert_eq!(
        fixture.codes(),
        [DiagnosticCode::Bc8006ConsumerFenceDebtExceeded]
    );
    fixture.write("docs/guide.md", "- An excerpt:\n\n  <!-- illustrative -->\n  ```rust,ignore\n  // Illustrative: not compiled.\n  let x = 1;\n  ```\n");
    assert!(fixture.diagnostics().is_empty());
}

#[test]
fn disabled_or_failure_tolerant_ci_steps_do_not_prove_doctests() {
    for control in ["if: false", "continue-on-error: true"] {
        let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
        fixture.write(".github/workflows/ci.yml", &format!("jobs:\n  docs:\n    steps:\n      - run: python scripts/ci/query_gate.py doc\n        {control}\n"));
        assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
        fixture.write(".github/workflows/ci.yml", &format!("jobs:\n  docs:\n    {control}\n    steps:\n      - run: python scripts/ci/query_gate.py doc\n"));
        assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    }
}

#[test]
fn explicit_compiled_targets_require_registered_reachable_active_source() {
    let fixture = Fixture::new(
        "<!-- compiled: owning.rs -->\n```rust\nfn example() {}\n```\n",
        0,
    );
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    fixture.write(CONFIG, &format!("{config}\n[[compiled_sources]]\nname='owning.rs'\ncrate_root='app'\ntarget='examples/owning.rs'\nsource='examples/owning/body.rs'\n"));
    fixture.write("app/Cargo.toml", "[package]\nname='app'\nversion='0.1.0'\nautoexamples=false\n[[example]]\nname='owning'\npath='examples/owning.rs'\n");
    fixture.write(
        "app/examples/owning.rs",
        "#[path=\"owning/body.rs\"] mod body;\nfn main() {}\n",
    );
    fixture.write(
        "app/examples/owning/body.rs",
        "// compiled-example: begin\nfn example() {}\n// compiled-example: end\n",
    );
    assert!(fixture.diagnostics().is_empty());
    fixture.write(
        "app/examples/owning.rs",
        "#[cfg(any())] #[path=\"owning/body.rs\"] mod body;\nfn main() {}\n",
    );
    assert!(fixture
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::Bc8008ConsumerDocMarker));
    fixture.write(
        "app/examples/owning.rs",
        "#[path=\"owning/body.rs\"] mod body;\nfn main() {}\n",
    );
    fixture.write(
        "app/Cargo.toml",
        "[package]\nname='app'\nversion='0.1.0'\nautoexamples=false\n",
    );
    assert!(fixture
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.code() == DiagnosticCode::Bc8008ConsumerDocMarker));
}
