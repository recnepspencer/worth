//! Public linking carries the relation, source entity and declaring fragment witness.
use std::process::Command;
#[test]
fn linking_a_foreign_extension_relation_reports_relation_entity_and_fragment() {
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let archive = |name: &str| {
        let mut paths: Vec<_> = std::fs::read_dir(&deps)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                let file = p.file_name().unwrap().to_string_lossy();
                file.starts_with(&format!("lib{name}-")) && file.ends_with(".rlib")
            })
            .collect();
        paths.sort_by_key(|p| std::fs::metadata(p).unwrap().modified().unwrap());
        paths.pop().unwrap()
    };
    let dir = std::env::temp_dir().join(format!(
        "worth-relation-extension-court-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("foreign_link.rs");
    std::fs::write(
        &source,
        r#"
use worth_query_testkit_decl::{fragment_fixture::*, fragment_extension_fixture::*};
use worth_query_execution::facade::primary_graph::{View, Writing, CandidateEntity};
fn foreign(view: &mut View<'_, Classifications, Writing<'_>>, item: &CandidateEntity<Item>) {
    view.link(item, Annotates, item).unwrap();
}
fn positive(view: &mut View<'_, Annotations, Writing<'_>>, item: &CandidateEntity<Item>) {
    view.link(item, Annotates, item).unwrap();
}
"#,
    )
    .unwrap();
    let mut command = Command::new("rustc");
    command
        .arg(&source)
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--emit=metadata",
            "--error-format=human",
        ])
        .arg("--out-dir")
        .arg(&dir)
        .arg("-L")
        .arg(format!("dependency={}", deps.display()));
    for name in ["worth_query_testkit_decl", "worth_query_execution"] {
        command
            .arg("--extern")
            .arg(format!("{name}={}", archive(name).display()));
    }
    let output = command.output().unwrap();
    let diagnostics = String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success(), "foreign extension link compiled");
    for expected in [
        "Annotates",
        "Item",
        "Classifications",
        "Annotates` is not an aspect or relation of entity/fragment `(Classifications, Item)`",
    ] {
        assert!(
            diagnostics.contains(expected),
            "missing {expected:?}: {diagnostics}"
        );
    }
    assert_eq!(diagnostics.matches("error[E").count(), 1, "{diagnostics}");
    println!("{diagnostics}");
}
