//! Facts, canonical claim references and navigation are exercised through real validation.
use super::*;

fn fact_fixture() -> Fixture {
    let text = "<!-- source-facts: choices begin -->\n<!-- Derived from `facade::Choice`; edit its source, then --write-guidance. -->\n\n| Variant | Shape |\n| --- | --- |\n| `First` | unit |\n<!-- source-facts: choices end -->\n";
    let fixture = Fixture::new(text, 0);
    fixture.write(
        "query/crates/example/src/lib.rs",
        "pub mod facade { pub enum Choice { First } }",
    );
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    fixture.write(CONFIG, &(config + "\n[[facts]]\nid='choices'\nmanifest='query/crates/example/Cargo.toml'\nitem='facade::Choice'\ndocument='docs/guide.md'\n"));
    fixture
}
#[test]
fn source_enum_change_is_detected_without_a_mirrored_variant_list() {
    let fixture = fact_fixture();
    assert!(fixture.diagnostics().is_empty());
    fixture.write(
        "query/crates/example/src/lib.rs",
        "pub mod facade { pub enum Choice { First, Second(u64) } }",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    let config = super::super::configuration::load(&fixture.root).unwrap();
    let writes = super::super::generated_regions::planned(&fixture.root, &config).unwrap();
    let expected = writes.values().next().unwrap();
    assert!(expected.contains("| `Second` | tuple (u64) |"));
    fixture.write("docs/guide.md", expected);
    assert!(fixture.diagnostics().is_empty());
}
#[test]
fn missing_duplicate_reversed_unknown_and_nested_fact_regions_refuse() {
    let fixture = fact_fixture();
    for text in [
        "no markers",
        "<!-- source-facts: choices begin -->\n<!-- source-facts: choices begin -->\n<!-- source-facts: choices end -->\n",
        "<!-- source-facts: choices end -->\n<!-- source-facts: choices begin -->\n",
        "<!-- source-facts: unknown begin -->\n",
        "<!-- source-facts: choices begin -->\n<!-- source-facts: choices end -->\n<!-- source-facts: choices end -->\n",
    ] {
        fixture.write("docs/guide.md", text);
        assert!(fixture.codes().contains(&DiagnosticCode::Bc8008ConsumerDocMarker), "{text}");
    }
}
#[test]
fn canonical_absence_and_references_need_actual_closed_source_knowledge() {
    let fixture = Fixture::new("# Task index\n\n<!-- facade-absence: missing -->\n### Gap: missing\n\nUnavailable.\n\n<!-- known-gap: missing -->\n[Canonical gap](guide.md#gap-missing)\n", 0);
    fixture.write("query/crates/example/src/lib.rs", "pub mod facade {}");
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    fixture.write(CONFIG, &(config + "\n[[absence_claims]]\nid='missing'\nmanifest='query/crates/example/Cargo.toml'\ncontainer='facade'\nname='Missing'\nkind='module-export'\n[navigation]\nindex='docs/guide.md'\nguides=[]\ncrate_roots=[]\n"));
    assert!(fixture.diagnostics().is_empty());
    fixture.write(
        "query/crates/example/src/lib.rs",
        "pub mod facade { pub struct Missing; }",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write(
        "query/crates/example/src/lib.rs",
        "pub mod facade { opaque!(); }",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write("docs/guide.md", "<!-- known-gap: unqualified -->\n");
    assert!(fixture
        .codes()
        .contains(&DiagnosticCode::Bc8008ConsumerDocMarker));
}
#[test]
fn navigation_requires_real_crate_docs_not_an_unrelated_string() {
    let fixture = Fixture::new(
        "# Task index\n\nUse the application guide to install and run a program.\n",
        0,
    );
    fixture.write("docs/build-an-application.md", "guide");
    fixture.write(
        "query/crates/example/src/lib.rs",
        "const LINK: &str = \"https://github.com/recnepspencer/worth/blob/master/docs/guide.md\";",
    );
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    fixture.write(CONFIG, &(config + "\n[navigation]\nindex='docs/guide.md'\nguides=[]\ncrate_roots=['query/crates/example/src/lib.rs']\n"));
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write("query/crates/example/src/lib.rs", "//! [Guide](https://github.com/recnepspencer/worth/blob/master/docs/build-an-application.md)\n//! [Index](https://github.com/recnepspencer/worth/blob/master/docs/guide.md)\n");
    assert!(fixture.diagnostics().is_empty());
}

#[test]
fn task_guides_reach_the_index_without_prescribing_editorial_content() {
    let fixture = Fixture::new("# Application tasks\n\nInstall and run a program.\n", 0);
    fixture.write("docs/build-an-application.md", "[Tasks](guide.md)\n");
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    let config = config.replace(
        "documents = ['docs/guide.md']",
        "documents = ['docs/guide.md', 'docs/build-an-application.md']",
    );
    fixture.write(CONFIG, &(config + "\n[navigation]\nindex='docs/guide.md'\nguides=['docs/build-an-application.md']\ncrate_roots=[]\n"));
    let debt = std::fs::read_to_string(fixture.root.join(DEBT)).unwrap();
    fixture.write(
        DEBT,
        &(debt + "\n[[documents]]\npath='docs/build-an-application.md'\nunclassified=0\n"),
    );
    assert!(fixture.diagnostics().is_empty());
    fixture.write("docs/build-an-application.md", "`[Tasks](guide.md)`\n");
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write("docs/other.md", "# Other\n");
    fixture.write("docs/build-an-application.md", "[Other](other.md)\n");
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
}

#[test]
fn region_replacement_preserves_prose_and_windows_line_endings() {
    let before = "unchanged prose\r\n<!-- source-facts: choices begin -->\r\nstale\r\n<!-- source-facts: choices end -->\r\nunchanged tail\r\n";
    let after = super::super::generated_regions::replace(before, "choices", "new\n").unwrap();
    assert_eq!(after, before.replace("stale", "new"));
}
#[test]
fn invalid_derived_source_prevents_document_writes() {
    let fixture = fact_fixture();
    fixture.write(
        "query/crates/example/src/lib.rs",
        "pub mod facade { pub enum Choice { First, Second } }",
    );
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    fixture.write(CONFIG, &("derive_skills=true\n".to_owned() + &config));
    let before = std::fs::read(fixture.root.join("docs/guide.md")).unwrap();
    assert!(super::super::write_guidance(&fixture.root).is_err());
    assert_eq!(
        std::fs::read(fixture.root.join("docs/guide.md")).unwrap(),
        before
    );
}

#[test]
fn active_region_offsets_ignore_literal_examples_and_refuse_nesting() {
    let replace = super::super::generated_regions::replace;
    let pair = "<!-- source-facts: choices begin -->\nstale\n<!-- source-facts: choices end -->\n";
    for literal in [
        format!("```text\n{pair}```\n"),
        format!(
            "`{}`\n`{}`\n",
            pair.lines().next().unwrap(),
            pair.lines().last().unwrap()
        ),
    ] {
        assert!(replace(&literal, "choices", "new\n").is_err());
        let both = format!("{literal}\n{pair}");
        let result = replace(&both, "choices", "new\n").unwrap();
        assert!(result.starts_with(&literal));
        assert!(result.ends_with(&pair.replace("stale", "new")));
    }
    let nested = "<!-- source-facts: choices begin -->\n<!-- source-facts: other begin -->\n<!-- source-facts: other end -->\n<!-- source-facts: choices end -->\n";
    assert!(replace(nested, "choices", "new\n").is_err());
    for newline in ["\n", "\r\n"] {
        let before = format!("prose\n{pair}tail\n").replace('\n', newline);
        assert_eq!(
            replace(&before, "choices", "new\n").unwrap(),
            before.replace("stale", "new")
        );
    }
}

#[test]
fn source_output_bindings_and_directory_conflicts_refuse_before_document_effects() {
    let fixture = fact_fixture();
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    let before = std::fs::read(fixture.root.join("docs/guide.md")).unwrap();
    for destination in [
        "query/crates/example/src/lib.rs",
        "skills/example/SKILL.md",
        "tools/boundary-check/snapshots/example.md",
    ] {
        fixture.write(
            destination,
            "<!-- source-facts: choices begin -->\n<!-- source-facts: choices end -->\n",
        );
        let changed = config
            .replace(
                "documents = ['docs/guide.md']",
                &format!("documents = ['docs/guide.md', '{destination}']"),
            )
            .replace(
                "document='docs/guide.md'",
                &format!("document='{destination}'"),
            );
        fixture.write(CONFIG, &changed);
        assert!(super::super::write_guidance(&fixture.root).is_err());
        assert_eq!(
            std::fs::read(fixture.root.join("docs/guide.md")).unwrap(),
            before
        );
    }
    fixture.write(
        "query/crates/example/src/lib.rs",
        "pub mod facade { pub enum Choice { First, Second } }",
    );
    fixture.write(CONFIG, &("derive_skills=true\n".to_owned() + &config));
    fixture.write("skills/example/SKILL.md", "source");
    fixture.write(".grok/skills/example", "not a directory");
    assert!(super::super::write_guidance(&fixture.root).is_err());
    assert_eq!(
        std::fs::read(fixture.root.join("docs/guide.md")).unwrap(),
        before
    );
}

#[cfg(unix)]
#[test]
fn document_alias_and_replaced_planned_parent_do_not_touch_other_targets() {
    use super::super::guidance_write::Plan;
    let fixture = fact_fixture();
    let before = std::fs::read(fixture.root.join("docs/guide.md")).unwrap();
    std::fs::rename(
        fixture.root.join("docs/guide.md"),
        fixture.root.join("docs/real.md"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        fixture.root.join("query/crates/example/src/lib.rs"),
        fixture.root.join("docs/guide.md"),
    )
    .unwrap();
    assert!(super::super::write_guidance(&fixture.root).is_err());
    std::fs::remove_file(fixture.root.join("docs/guide.md")).unwrap();
    std::fs::rename(
        fixture.root.join("docs/real.md"),
        fixture.root.join("docs/guide.md"),
    )
    .unwrap();
    let mut writes = std::collections::BTreeMap::new();
    writes.insert(fixture.root.join("docs/guide.md"), "changed".into());
    let plan = Plan::new(&fixture.root, writes, None).unwrap();
    std::fs::rename(
        fixture.root.join("docs"),
        fixture.root.join("original-docs"),
    )
    .unwrap();
    fixture.write("other/guide.md", "unrelated");
    std::os::unix::fs::symlink(fixture.root.join("other"), fixture.root.join("docs")).unwrap();
    assert!(plan.apply().is_err());
    assert_eq!(
        std::fs::read(fixture.root.join("other/guide.md")).unwrap(),
        b"unrelated"
    );
    assert_eq!(
        std::fs::read(fixture.root.join("original-docs/guide.md")).unwrap(),
        before
    );
    std::fs::remove_file(fixture.root.join("docs")).unwrap();
}

#[test]
fn gap_links_require_the_actual_named_heading() {
    let valid = "# Task index\n\n<!-- facade-absence: missing -->\n### Gap: missing\n\nUnavailable.\n\n<!-- known-gap: missing -->\n[Gap](guide.md#gap-missing)\n";
    let fixture = Fixture::new(valid, 0);
    fixture.write("query/crates/example/src/lib.rs", "pub mod facade {}");
    let config = std::fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    fixture.write(CONFIG, &(config + "\n[[absence_claims]]\nid='missing'\nmanifest='query/crates/example/Cargo.toml'\ncontainer='facade'\nname='Missing'\nkind='module-export'\n[navigation]\nindex='docs/guide.md'\nguides=[]\ncrate_roots=[]\n"));
    assert!(fixture.diagnostics().is_empty());
    for invalid in [
        valid.replace("### Gap: missing\n", ""),
        valid.replace("[Gap](guide.md#gap-missing)", "marker without a link"),
        valid.replace("### Gap: missing", "### Gap: other"),
        valid.replace("Unavailable.", "Unavailable.\n\n### Gap: missing"),
    ] {
        fixture.write("docs/guide.md", &invalid);
        assert!(!fixture.diagnostics().is_empty(), "{invalid}");
    }
}

#[test]
fn confined_writer_publishes_complete_files_and_refuses_changed_observations() {
    use super::super::guidance_write::Plan;
    let fixture = fact_fixture();
    let path = fixture.root.join("docs/guide.md");
    let mut writes = std::collections::BTreeMap::new();
    writes.insert(path.clone(), "complete replacement".into());
    Plan::new(&fixture.root, writes.clone(), None)
        .unwrap()
        .apply()
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"complete replacement");
    let plan = Plan::new(&fixture.root, writes, None).unwrap();
    fixture.write("docs/guide.md", "new author edit");
    assert!(plan.apply().is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"new author edit");
}

#[test]
fn literal_only_fact_markers_refuse_guidance_without_writes() {
    let fixture = fact_fixture();
    for literal in [
        "```text\n<!-- source-facts: choices begin -->\nstale\n<!-- source-facts: choices end -->\n```\n",
        "`<!-- source-facts: choices begin -->`\n`<!-- source-facts: choices end -->`\n",
    ] {
        fixture.write("docs/guide.md", literal);
        assert!(super::super::write_guidance(&fixture.root).is_err());
        assert_eq!(std::fs::read(fixture.root.join("docs/guide.md")).unwrap(), literal.as_bytes());
    }
}
