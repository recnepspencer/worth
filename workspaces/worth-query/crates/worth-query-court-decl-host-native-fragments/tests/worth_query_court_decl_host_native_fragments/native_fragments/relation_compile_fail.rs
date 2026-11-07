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
use worth_query_host::facade::{
    declaration::application_schema::fragment::{Fragment, Entity, Extend, Relation, NoBindings},
    primary_graph::{View, Writing, CandidateEntity},
};
struct Item;
impl Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = ();
    type Relations = ();
}
struct Annotates;
impl Relation for Annotates {
    const NAME: &'static str = "annotates";
    type To = Item;
}
struct Classifications;
impl Fragment for Classifications {
    const NAME: &'static str = "classifications";
    type Bindings = NoBindings;
    type Members = (Item,);
}
struct Annotations;
impl Fragment for Annotations {
    const NAME: &'static str = "annotations";
    type Bindings = NoBindings;
    type Members = (Extend<Item, (), (Annotates,)>,);
}
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
    // A single facade supplies both the fixture and View types. Independent
    // newest-library choices can combine incompatible workspace build graphs.
    command.arg("--extern").arg(format!(
        "worth_query_host={}",
        archive("worth_query_host").display()
    ));
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
