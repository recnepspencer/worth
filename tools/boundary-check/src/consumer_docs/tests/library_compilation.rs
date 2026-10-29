//! Library-owned document regions use the configuration of their real library unit.
use super::*;

fn library_fixture() -> Fixture {
    let fixture = Fixture::new(
        "<!-- compiled: library.rs -->\n```rust\nfn example() {}\n```\n",
        0,
    );
    let mut config = fs::read_to_string(fixture.root.join(CONFIG)).unwrap();
    config.push_str("\n[[compiled_sources]]\nname='library.rs'\ncrate_root='world'\ntarget='src/lib.rs'\nsource='src/example.rs'\n");
    fixture.write(CONFIG, &config);
    fixture.write(
        "world/Cargo.toml",
        "[package]\nname='world'\nversion='0.1.0'\n[lib]\ntest=false\n",
    );
    fixture.write("world/src/lib.rs", "mod example;\n");
    fixture.write(
        "world/src/example.rs",
        "// compiled-example: begin\nfn example() {}\n// compiled-example: end\n",
    );
    fixture
}

#[test]
fn library_regions_require_their_actual_normal_compilation_conditions() {
    let fixture = library_fixture();
    for (condition, accepted) in [("not(test)", true), ("test", false), ("any()", false)] {
        fixture.write(
            "world/src/lib.rs",
            &format!("#[cfg({condition})]\nmod example;\n"),
        );
        assert_eq!(
            fixture.diagnostics().is_empty(),
            accepted,
            "module {condition}"
        );
        fixture.write("world/src/lib.rs", "mod example;\n");
        fixture.write("world/src/example.rs", &format!("// compiled-example: begin\n#[cfg({condition})]\nfn example() {{}}\n// compiled-example: end\n"));
        assert_eq!(
            fixture.diagnostics().is_empty(),
            accepted,
            "item {condition}"
        );
        fixture.write(
            "world/src/example.rs",
            "// compiled-example: begin\nfn example() {}\n// compiled-example: end\n",
        );
    }
}

#[test]
fn library_targets_require_registration_and_follow_explicit_source_paths() {
    let fixture = library_fixture();
    fixture.write(
        "world/Cargo.toml",
        "[package]\nname='world'\nversion='0.1.0'\nautolib=false\n",
    );
    assert_eq!(fixture.codes(), [DiagnosticCode::Bc8008ConsumerDocMarker]);
    fixture.write("world/Cargo.toml", "[package]\nname='world'\nversion='0.1.0'\nautolib=false\n[lib]\npath='world.rs'\ntest=false\n");
    fixture.write(
        "world/world.rs",
        "#[path=\"src/example.rs\"] mod example;\n",
    );
    let config = fs::read_to_string(fixture.root.join(CONFIG))
        .unwrap()
        .replace("target='src/lib.rs'", "target='world.rs'");
    fixture.write(CONFIG, &config);
    assert!(fixture.diagnostics().is_empty());
}
