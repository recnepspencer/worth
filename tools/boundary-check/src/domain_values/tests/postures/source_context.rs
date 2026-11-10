//! Namespace evidence for real mounted sources, imports and immutable includes.
use super::super::{check_sources, roots, Files};
use super::{canonical, policy};

#[test]
fn every_actual_mount_of_a_registered_binding_must_be_reference_free() {
    let binding = canonical("Imported", "of");
    let registry = policy("domain").replace("app/lib.rs", "app/shared.rs");
    let mut files = Files::from([
        ("app/lib.rs".into(), "mod first; mod second;".into()),
        (
            "app/first.rs".into(),
            "pub type Imported = u64; #[path=\"shared.rs\"] mod binding;".into(),
        ),
        (
            "app/second.rs".into(),
            "pub type Imported = u32; #[path=\"shared.rs\"] mod binding;".into(),
        ),
        (
            "app/shared.rs".into(),
            format!("use super::Imported; {binding}"),
        ),
        ("bindings.toml".into(), registry),
    ]);
    assert_eq!(check_sources(&files, roots(), "bindings.toml"), Ok(()));
    files.insert(
        "app/second.rs".into(),
        "pub type Imported = EntityRef; #[path=\"shared.rs\"] mod binding;".into(),
    );
    let error = check_sources(&files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("EntityRef"), "{error}");
}

#[test]
fn glob_derived_namespaces_are_checked_instead_of_trusted_or_blanket_refused() {
    let binding = canonical("String", "of").replace("struct Leaf", "#[derive(Clone)] struct Leaf");
    let mut files = Files::from([
        (
            "app/lib.rs".into(),
            format!("use exports::*; mod exports {{ pub struct Other; }} {binding}"),
        ),
        ("bindings.toml".into(), policy("domain")),
    ]);
    assert_eq!(check_sources(&files, roots(), "bindings.toml"), Ok(()));
    for exports in [
        "pub struct Clone;",
        "pub use unavailable::Clone;",
        "pub use super::*; pub use unavailable::Clone;",
    ] {
        files.insert(
            "app/lib.rs".into(),
            format!("use exports::*; mod exports {{ {exports} }} {binding}"),
        );
        assert!(check_sources(&files, roots(), "bindings.toml").is_err());
    }
    let qualified = binding.replace("derive(Clone)", "derive(serde::Serialize)");
    files.insert(
        "app/lib.rs".into(),
        format!("use exports::*; mod exports {{ pub mod serde {{}} }} {qualified}"),
    );
    assert!(check_sources(&files, roots(), "bindings.toml").is_err());
    files.insert(
        "app/lib.rs".into(),
        format!("#[macro_use] extern crate unavailable; {binding}"),
    );
    assert!(check_sources(&files, roots(), "bindings.toml").is_err());
}

#[test]
fn literal_includes_keep_real_namespaces_and_cannot_hide_a_reference() {
    let binding = canonical("String", "of").replace("struct Leaf", "#[derive(Clone)] struct Leaf");
    let mut files = Files::from([
        (
            "app/lib.rs".into(),
            format!("mod exports {{ include!(\"included.rs\"); }} use exports::*; {binding}"),
        ),
        ("app/exports/included.rs".into(), "pub struct Other;".into()),
        ("app/included.rs".into(), "pub struct Other;".into()),
        ("bindings.toml".into(), policy("domain")),
    ]);
    assert_eq!(check_sources(&files, roots(), "bindings.toml"), Ok(()));
    // Inline modules resolve include! relative to the containing source file.
    files.insert(
        "app/included.rs".into(),
        "pub struct String { hidden: EntityRef }".into(),
    );
    let error = check_sources(&files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("EntityRef"), "{error}");
    files.insert("app/included.rs".into(), "mod redirected;".into());
    assert!(check_sources(&files, roots(), "bindings.toml").is_err());
    files.insert(
        "app/lib.rs".into(),
        format!("include!(concat!(\"included\", \".rs\")); {binding}"),
    );
    assert!(check_sources(&files, roots(), "bindings.toml").is_err());
}

#[test]
fn binary_root_identity_preserves_fully_qualified_forbidden_type_paths() {
    use crate::{
        config::{Code, Metadata, Package, Target},
        inputs::Inputs,
    };
    let source = canonical("u64", "of");
    let registry = policy("domain")
        .replace("app/lib.rs", "app/main.rs")
        .replace("['GraphKey', 'ObjectName']", "['app::Leaf']");
    let mut input = Inputs::fixture(
        &[("app/main.rs", &source), ("bindings.toml", &registry)],
        "",
    );
    input.graphs.insert(
        "Cargo.toml".into(),
        Metadata {
            packages: vec![Package {
                targets: vec![Target {
                    name: "app".into(),
                    kind: vec!["bin".into()],
                    src_path: "app/main.rs".into(),
                }],
                ..Default::default()
            }],
            ..Default::default()
        },
    );
    let diagnostics = crate::domain_values::check_registry(&input, "bindings.toml");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Code::DomainValue);
    assert!(diagnostics[0]
        .message
        .contains("reference/key/name forbidden inside domain value: app::Leaf"));
}

#[test]
fn extern_prelude_crates_are_not_implicit_exports_of_every_glob_namespace() {
    let binding = canonical("u64", "of").replace(
        "struct Leaf",
        "#[derive(worth_foundational::facade::WorthValue)] struct Leaf",
    );
    let mut files = Files::from([
        (
            "app/lib.rs".into(),
            format!("use exports::*; mod exports {{ pub struct Other; }} {binding}"),
        ),
        ("foundational/lib.rs".into(), "pub mod facade {}".into()),
        ("bindings.toml".into(), policy("domain")),
    ]);
    let mut roots = roots();
    roots.insert("worth_foundational".into(), "foundational/lib.rs".into());
    assert_eq!(
        check_sources(&files, roots.clone(), "bindings.toml"),
        Ok(())
    );
    files.insert(
        "app/lib.rs".into(),
        format!("use exports::*; mod exports {{ pub mod worth_foundational {{}} }} {binding}"),
    );
    assert!(check_sources(&files, roots, "bindings.toml").is_err());
}

#[test]
fn glob_modules_shadow_relative_standard_paths_but_not_absolute_paths() {
    let roots = roots();
    let mut files = Files::from([("bindings.toml".into(), policy("domain"))]);
    let path = "std::string::String";
    let exports = "pub mod std { pub mod string { pub struct String { hidden: EntityRef } } }";
    for absolute in [false, true] {
        let value = if absolute {
            format!("::{path}")
        } else {
            path.to_owned()
        };
        let binding =
            canonical("u64", "of").replace("type Value = Leaf;", &format!("type Value = {value};"));
        files.insert(
            "app/lib.rs".into(),
            format!("use exports::*; mod exports {{ {exports} }} {binding}"),
        );
        let result = check_sources(&files, roots.clone(), "bindings.toml");
        if absolute {
            assert_eq!(result, Ok(()));
        } else {
            let error = result.unwrap_err().join("\n");
            assert!(error.contains("EntityRef"), "{error}");
        }
    }
}

#[test]
fn duplicate_module_declarations_cannot_reuse_a_cached_mount() {
    let binding = canonical("Imported", "of");
    let registry = policy("domain").replace("app/lib.rs", "app/shared.rs");
    let files = Files::from([
        ("app/lib.rs".into(), "#[cfg(any())] #[path=\"first.rs\"] mod mount; #[cfg(all())] #[path=\"second.rs\"] mod mount;".into()),
        ("app/first.rs".into(), "pub type Imported = u64; #[path=\"shared.rs\"] mod binding;".into()),
        ("app/second.rs".into(), "pub type Imported = EntityRef; #[path=\"shared.rs\"] mod binding;".into()),
        ("app/shared.rs".into(), format!("use super::Imported; {binding}")),
        ("bindings.toml".into(), registry),
    ]);
    let error = check_sources(&files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("ambiguous module"), "{error}");
}

#[test]
fn unrelated_conditional_modules_do_not_own_the_binding_namespace() {
    let binding = canonical("u64", "of");
    let registry = policy("domain").replace("app/lib.rs", "app/values.rs");
    let mut files = Files::from([
        ("app/lib.rs".into(), "mod values; #[cfg(any())] #[path=\"first.rs\"] mod installed; #[cfg(all())] #[path=\"second.rs\"] mod installed;".into()),
        ("app/first.rs".into(), "pub type Imported = u64;".into()),
        ("app/second.rs".into(), "pub type Imported = EntityRef;".into()),
        ("app/values.rs".into(), binding.clone()),
        ("bindings.toml".into(), registry),
    ]);
    assert_eq!(check_sources(&files, roots(), "bindings.toml"), Ok(()));
    files.insert(
        "app/values.rs".into(),
        binding.replace(
            "type Value = Leaf;",
            "type Value = crate::installed::Imported;",
        ),
    );
    let error = check_sources(&files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("ambiguous"), "{error}");
}
