//! The joined roster opens only the same sealed case-membership witnesses.
use std::{path::PathBuf, process::Command};

#[test]
fn foreign_case_in_joined_one_of_is_a_compile_error_with_a_lawful_control() {
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let mut archives: Vec<_> = std::fs::read_dir(&deps)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.starts_with("libworth_query_host-") && name.ends_with(".rlib")
        })
        .collect();
    archives.sort_by_key(|path| std::fs::metadata(path).unwrap().modified().unwrap());
    let host = archives.pop().unwrap();
    let directory = std::env::temp_dir().join(format!(
        "worth-joined-case-membership-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/worth_query_court_decl_host_native_fragments/native_fragments/large_one_of/schema.rs");
    let base = format!(
        r#"
use worth_query_host::facade::declaration::application_schema::{{fragment::*, *}};
#[path = {fixture:?}]
mod schema;
#[path = {principal:?}]
mod principal;
use schema::*;
fn lawful() {{
    let _ = Fixture::declaration().unwrap();
    let _ = aspect(ROOT, ITEM, Case01);
    let _ = aspect(ROOT, ITEM, Case32);
}}
"#,
        fixture = fixture.to_str().unwrap(),
        principal = fixture.with_file_name("principal.rs").to_str().unwrap()
    );
    let compile = |name: &str, source: &str| {
        let path = directory.join(format!("{name}.rs"));
        std::fs::write(&path, source).unwrap();
        Command::new("rustc")
            .arg(path)
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--error-format=human",
            ])
            .arg("--out-dir")
            .arg(&directory)
            .arg("-L")
            .arg(format!("dependency={}", deps.display()))
            .arg("--extern")
            .arg(format!("worth_query_host={}", host.display()))
            .output()
            .unwrap()
    };
    let lawful = compile("lawful", &base);
    assert!(
        lawful.status.success(),
        "{}",
        String::from_utf8_lossy(&lawful.stderr)
    );
    let denied = compile(
        "foreign",
        &(base
            + r#"
struct ForeignCase;
impl Aspect for ForeignCase {
    const NAME: &'static str = "foreign_case";
    type Row = u64;
}
fn foreign() { let _ = aspect(ROOT, ITEM, ForeignCase); }
"#),
    );
    let message = String::from_utf8(denied.stderr).unwrap();
    assert!(
        !denied.status.success(),
        "a foreign case opened a joined witness"
    );
    for name in [
        "ForeignCase",
        "Root",
        "Entry",
        "is not an aspect or relation of entity/fragment",
    ] {
        assert!(message.contains(name), "missing {name}: {message}");
    }
    assert_eq!(message.matches("error[E").count(), 1, "{message}");
}
