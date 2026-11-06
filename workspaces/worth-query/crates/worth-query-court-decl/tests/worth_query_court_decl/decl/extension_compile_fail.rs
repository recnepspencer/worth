//! The rendered diagnostic identifies the rejected member and its declaration path.
use std::process::Command;
#[test]
fn extension_selection_and_required_members_fail_at_the_public_boundary() {
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
    let dir = std::env::temp_dir().join(format!("worth-extension-court-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("extension_refusals.rs");
    std::fs::write(&source, r#"
use worth_query_testkit_decl::fragment_fixture::*;
use worth_query_testkit_decl::fragment_extension_fixture::*;
fn foreign() { let _ = aspect(Annotations, Item, Classified); }
struct Required;
impl Fragment for Required {
    const NAME: &'static str = "required";
    type Bindings = Bindings;
    type Members = (Extend<Item, (Annotation,)>,);
}
struct RequiredGroup;
impl Fragment for RequiredGroup {
    const NAME: &'static str = "required_group";
    type Bindings = Bindings;
    type Members = (Extend<Item, (Exclusive<Classification>,)>,);
}
fn positive() { let _: AspectSelection<Annotations, Item, Annotation> = aspect(Annotations, Item, Annotation); }
"#).unwrap();
    let output = Command::new("rustc")
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
        .arg(format!("dependency={}", deps.display()))
        .arg("--extern")
        .arg(format!(
            "worth_query_testkit_decl={}",
            archive("worth_query_testkit_decl").display()
        ))
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "invalid extension declarations compiled"
    );
    let diagnostics = String::from_utf8(output.stderr).unwrap();
    for expected in [
        "Classified",
        "Item",
        "Annotations",
        "not an aspect",
        "required member in Extend",
        "Exclusive",
        "Classification",
        "Annotation",
    ] {
        assert!(
            diagnostics.contains(expected),
            "missing {expected:?}: {diagnostics}"
        );
    }
    assert_eq!(
        diagnostics.matches("error[E").count(),
        3,
        "unexpected compiler refusal: {diagnostics}"
    );
}
