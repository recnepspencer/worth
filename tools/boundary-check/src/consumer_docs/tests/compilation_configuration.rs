//! Compilation claims must survive Rust configuration at their actual owners.
use super::*;

fn compiled_fixture() -> Fixture {
    let fixture = Fixture::new(
        "<!-- compiled: example.rs -->\n```rust\nfn example() {}\n```\n",
        0,
    );
    fixture.source("// compiled-example: begin\nfn example() {}\n// compiled-example: end\n");
    fixture
}

#[test]
fn inactive_and_unsupported_modules_cannot_claim_native_test_compilation() {
    let fixture = compiled_fixture();
    for condition in [
        "any()",
        "not(test)",
        "target_family = \"wasm\"",
        "not(unknown_condition)",
    ] {
        fixture.write(
            &format!("{SOURCE}/lib.rs"),
            &format!("#[cfg({condition})]\nmod example;\n"),
        );
        assert_eq!(
            fixture.codes(),
            [DiagnosticCode::Bc8008ConsumerDocMarker],
            "{condition}"
        );
    }
    for condition in ["test", "all(test, not(target_family = \"wasm\"))"] {
        fixture.write(
            &format!("{SOURCE}/lib.rs"),
            &format!("#[cfg({condition})]\nmod example;\n"),
        );
        assert!(fixture.diagnostics().is_empty(), "{condition}");
    }
}

#[test]
fn inactive_file_and_ancestor_conditions_cannot_be_lost_by_the_inventory() {
    let fixture = compiled_fixture();
    fixture.write(
        &format!("{SOURCE}/lib.rs"),
        "#![cfg(any())]\nmod example;\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write(&format!("{SOURCE}/lib.rs"), "mod outer;\n");
    fixture.write(
        &format!("{SOURCE}/outer.rs"),
        "#![cfg(any())]\n#[path = \"example.rs\"]\nmod example;\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write(
        &format!("{SOURCE}/outer.rs"),
        "#![cfg(test)]\n#[path = \"example.rs\"]\nmod example;\n",
    );
    assert!(fixture.diagnostics().is_empty());
}

#[test]
fn inactive_items_in_and_around_the_region_are_refused() {
    let fixture = compiled_fixture();
    for source in [
        "// compiled-example: begin\n#[cfg(any())]\nfn example() {}\n// compiled-example: end\n",
        "#[cfg(any())]\nfn example() {\n// compiled-example: begin\nlet x = 1;\n// compiled-example: end\n}\n",
        "// compiled-example: begin\n#[cfg_attr(all(), cfg(any()))]\nfn example() {}\n// compiled-example: end\n",
        "#[cfg(any())]\nmod example {\n// compiled-example: begin\nfn example() {}\n// compiled-example: end\n}\n",
        "struct Example;\nimpl Example {\n#[cfg(any())]\nconst VALUE: u32 = {\n// compiled-example: begin\n1\n// compiled-example: end\n};\n}\n",
    ] {
        fixture.write(&format!("{SOURCE}/example.rs"), source);
        assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker], "{source}");
    }
    fixture.write(&format!("{SOURCE}/example.rs"), "#[cfg(test)]\nfn example() {\n// compiled-example: begin\nlet x = 1;\n// compiled-example: end\n}\n");
    assert!(fixture.diagnostics().is_empty());
}

#[test]
fn region_bounds_follow_newlines_regardless_of_marker_indentation() {
    let fixture = compiled_fixture();
    for indent in ["", "    "] {
        for body in [
            "fn example() { unresolved(); }",
            "fn example() {\n    unresolved();\n}",
        ] {
            fixture.source(&format!(
                "#[cfg(any())]\n{indent}// compiled-example: begin\n{indent}{body}\n{indent}// compiled-example: end\n"
            ));
            assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
        }
        for body in ["fn example() {}", "fn example() {\n    let x = 1;\n}"] {
            fixture.source(&format!(
                "#[cfg(test)]\n{indent}// compiled-example: begin\n{indent}{body}\n{indent}// compiled-example: end\n#[cfg(any())]\nfn outside() {{ unresolved(); }}\n"
            ));
            assert!(fixture.diagnostics().is_empty());
        }
    }
}

#[test]
fn cfg_attr_cannot_suppress_the_doctest_library() {
    let fixture = Fixture::new("<!-- doctest: example -->\n```rust\nlet x = 1;\n```\n", 0);
    let inclusion = "#![doc = include_str!(\"../../../../docs/guide.md\")]\n";
    fixture.write(
        "query/crates/example/src/lib.rs",
        &format!("#![cfg_attr(all(), cfg(any()))]\n{inclusion}"),
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write("query/crates/example/src/lib.rs", inclusion);
    assert!(fixture.diagnostics().is_empty());
}
