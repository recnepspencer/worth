use super::*;
#[test]
fn documented_enum_rows_detect_payload_changes_as_well_as_variants() {
    let mut input = Inputs::fixture(&[("src/model.rs", "pub enum Outcome { Ready(u32), Empty }") , ("doc.md", "<!-- source-facts: out begin -->\n| `Ready` | tuple (u32) |\n| `Empty` | unit |\n<!-- source-facts: out end -->")], "");
    let config = toml::from_str("[[facts]]\nid='out'\nitem='Outcome'\ndocument='doc.md'").unwrap();
    assert!(source_facts::check(&input, &config).is_empty());
    input.files.insert(
        "src/model.rs".into(),
        "pub enum Outcome { Ready(u64), Empty }".into(),
    );
    assert_eq!(source_facts::check(&input, &config).len(), 1);
}
#[test]
fn compiled_regions_compare_exact_code_and_require_unique_delimiters() {
    let source = "// compiled-example: begin\ncall();\n// compiled-example: end\n";
    assert!(region_matches(source, "call();\n"));
    assert!(!region_matches(source, "other();\n"));
    assert!(!region_matches("", ""));
}
#[test]
fn relative_links_require_targets_and_unique_heading_anchors() {
    let input = Inputs::fixture(&[("docs/a.md", "# Heading\n# Heading\n")], "");
    assert!(link(&input, "docs/a.md", "#heading-1").is_ok());
    assert!(link(&input, "docs/a.md", "missing.md").is_err());
    assert!(link(&input, "docs/a.md", "#absent").is_err());
}
#[test]
fn shortcut_reference_links_check_defined_targets_without_treating_definitions_as_links() {
    let mut input = Inputs::fixture(
        &[
            (
                "tools/boundary-check/config/consumer-docs.toml",
                "documents=['docs/a.md']",
            ),
            (
                "docs/a.md",
                "[label]\n[label]: exists.md\n[unused]: missing.md",
            ),
            ("docs/exists.md", ""),
        ],
        "",
    );
    assert!(check(&input).is_empty());
    input
        .files
        .insert("docs/a.md".into(), "[label]\n[label]: missing.md".into());
    let failures = check(&input);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].code, Code::DocumentLink);
    assert_eq!(failures[0].subject, "docs/a.md:1");
}
#[test]
fn a_compiled_marker_requires_a_following_code_region() {
    let mut input = Inputs::fixture(
        &[
            (
                "tools/boundary-check/config/consumer-docs.toml",
                "documents=['docs/a.md']",
            ),
            ("docs/a.md", "Ordinary text."),
        ],
        "",
    );
    assert!(check(&input).is_empty());
    input
        .files
        .insert("docs/a.md".into(), "<!-- compiled: demo.rs -->".into());
    assert_eq!(check(&input)[0].code, Code::DocumentRegion);
}

#[test]
fn compiled_markers_cannot_hide_orphans_or_malformed_directives() {
    let mut input = Inputs::fixture(&[
        ("tools/boundary-check/config/consumer-docs.toml", "documents=['docs/a.md']\n[[compiled_sources]]\nname='demo.rs'\ncrate_root='demo'\nsource='lib.rs'"),
        ("demo/lib.rs", "// compiled-example: begin\ncall();\n// compiled-example: end\n"),
        ("docs/a.md", "<!-- compiled: demo.rs -->\n```rust\ncall();\n```"),
    ], "");
    assert!(check(&input).is_empty());
    for broken in [
        "<!-- compiled: missing.rs -->\n<!-- compiled: demo.rs -->\n```rust\ncall();\n```",
        "<!-- compiled: demo.rs-->\n```rust\ncall();\n```",
    ] {
        input.files.insert("docs/a.md".into(), broken.into());
        let failures = check(&input);
        assert_eq!(failures.len(), 1, "{broken}");
        assert_eq!(failures[0].code, Code::DocumentRegion);
    }
    let code = "let example = r#\"\n```\n\"#;\n";
    input.files.insert(
        "demo/lib.rs".into(),
        format!("// compiled-example: begin\n{code}// compiled-example: end\n"),
    );
    input.files.insert(
        "docs/a.md".into(),
        format!("<!-- compiled: demo.rs -->\n````rust\n{code}`````"),
    );
    assert!(check(&input).is_empty());
}
#[test]
fn multiline_relative_links_are_checked_outside_fenced_examples() {
    let mut input = Inputs::fixture(
        &[
            (
                "tools/boundary-check/config/consumer-docs.toml",
                "documents=['docs/a.md']",
            ),
            ("docs/a.md", "[label](\nexists.md\n)"),
            ("docs/exists.md", ""),
        ],
        "",
    );
    assert!(check(&input).is_empty());
    input
        .files
        .insert("docs/a.md".into(), "[label](\nmissing.md\n)".into());
    let failures = check(&input);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].code, Code::DocumentLink);
    assert_eq!(failures[0].subject, "docs/a.md:1");
    input.files.insert(
        "docs/a.md".into(),
        "```text\n[label](\nmissing.md\n)\n```".into(),
    );
    assert!(check(&input).is_empty());
}
