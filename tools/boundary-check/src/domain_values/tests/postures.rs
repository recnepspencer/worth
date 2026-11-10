//! Classification is mandatory; only reviewed diagnostics skip the field walk.
use super::{registration, run};
mod source_context;

fn canonical(value: &str, constructor: &str) -> String {
    format!(
        r#"
        struct Leaf {{ value: {value} }}
        impl BindingTrait for Binding {{
            type Value = Leaf;
            fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {{
                Some(ApplicationCanonicalValueContract::{constructor}::<String>(BOUNDS))
            }}
        }}
    "#
    )
}

fn policy(posture: &str) -> String {
    registration("Binding").replace("posture='domain'", &format!("posture='{posture}'"))
}

#[test]
fn canonical_constructor_bindings_require_registration_of_the_actual_value() {
    for constructor in ["of", "delegated"] {
        let source = canonical("u64", constructor);
        let error = run(&source, "binding=[]").unwrap_err().join("\n");
        assert!(error.contains("app/lib.rs::Binding"), "{error}");
        assert!(error.contains("needs exactly one"), "{error}");
        assert_eq!(run(&source, &policy("exception")), Ok(()));
        // Canonical discovery does not depend on the domain marker's truth.
        let source = source.replace(
            "type Value = Leaf;",
            "type Value = Leaf; const DOMAIN_VALUE: bool = false;",
        );
        assert!(run(&source, "binding=[]").is_err());
        assert_eq!(run(&source, &policy("exception")), Ok(()));
        let aliased = source.replace("ApplicationCanonicalValueContract::", "ContractAlias::");
        assert!(run(&aliased, "binding=[]").is_err());
        let generated = format!("macro_rules! generated {{ () => {{ {source} }} }}");
        assert!(run(&generated, "binding=[]")
            .unwrap_err()
            .join("\n")
            .contains("unknown macro"));
    }
    // Creating an isolated contract is not declaring a binding.
    assert_eq!(
        run(
            "fn contract() { ApplicationCanonicalValueContract::of::<u64>(BOUNDS); }",
            "binding=[]"
        ),
        Ok(())
    );
}

#[test]
fn exceptions_walk_the_associated_value_even_when_the_codec_is_reference_free() {
    for constructor in ["of", "delegated"] {
        let source = canonical("Option<Box<Nested>>", constructor)
            + "struct Nested { reference: EntityRef }";
        for posture in ["domain", "exception"] {
            let error = run(&source, &policy(posture)).unwrap_err().join("\n");
            assert!(error.contains("app/lib.rs::Binding"), "{error}");
            assert!(error.contains("reference/key/name forbidden"), "{error}");
            assert!(error.contains("EntityRef"), "{error}");
        }
    }
}

#[test]
fn diagnostics_skip_only_the_reference_walk_and_require_a_nonempty_reason() {
    let source = canonical("EntityRef", "delegated");
    let registry = policy("diagnostic");
    assert_eq!(run(&source, &registry), Ok(()));
    for invalid in [
        registry.replace("reason='reference-free fixture value'\n", ""),
        registry.replace("reason='reference-free fixture value'", "reason='   '"),
        registry.replace("reason='reference-free fixture value'", "reason=17"),
    ] {
        let error = run(&source, &invalid).unwrap_err().join("\n");
        assert!(error.contains("reason"), "{error}");
        assert!(error.contains("Binding"), "{error}");
    }
    let stale = registry.replace("binding = 'Binding'", "binding = 'Absent'");
    assert!(run(&source, &stale)
        .unwrap_err()
        .join("\n")
        .contains("stale registration"));
    let local = format!("fn local() {{ {source} }}");
    assert!(run(&local, &registry)
        .unwrap_err()
        .join("\n")
        .contains("local scope"));
}

#[test]
fn sdk_registrations_require_posture_and_reject_unknown_postures() {
    let sdk = include_str!("../../../config/domain-values.toml");
    let mut files = super::Files::from([
        ("bindings.toml".into(), sdk.to_owned()),
        ("workspaces/worth-query/crates/worth-query-certification/fixtures/consumer_entry/Cargo.toml".into(), "[workspace]".into()),
    ]);
    let registrations = super::super::registrations(&files, "bindings.toml").unwrap();
    let posture_lines: Vec<_> = sdk
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.starts_with("posture = ").then_some(index))
        .collect();
    assert_eq!(posture_lines.len(), registrations.len());
    for (line, registration) in posture_lines.into_iter().zip(registrations) {
        // Remove the semantic field irrespective of checkout line endings.
        let missing_posture = sdk
            .lines()
            .enumerate()
            .filter_map(|(index, text)| (index != line).then_some(text))
            .collect::<Vec<_>>()
            .join("\n");
        files.insert("bindings.toml".into(), missing_posture);
        let error = super::super::registrations(&files, "bindings.toml")
            .err()
            .unwrap();
        assert!(error.contains("missing/nonliteral posture"), "{error}");
        assert!(error.contains(&registration.binding), "{error}");
    }
    let source = canonical("u64", "of");
    for invalid in [
        policy("unclassified"),
        policy("exception").replace("posture='exception'\n", ""),
    ] {
        let error = run(&source, &invalid).unwrap_err().join("\n");
        assert!(error.contains("posture"), "{error}");
    }
}

#[test]
fn binary_crate_roots_own_registered_claims_and_refuse_unregistered_claims() {
    use crate::{
        config::{Code, Metadata, Package, Target},
        inputs::Inputs,
    };
    let source = canonical("u64", "of");
    let registry = policy("domain").replace("app/lib.rs", "app/values.rs");
    let mut input = Inputs::fixture(
        &[
            ("app/main.rs", "mod values;"),
            ("app/values.rs", &source),
            ("bindings.toml", &registry),
        ],
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
    assert!(super::super::check_registry(&input, "bindings.toml").is_empty());
    // A same-named library cannot replace the binary's module tree.
    input
        .files
        .insert("app/lib.rs".into(), "pub struct Other;".into());
    input.graphs.get_mut("Cargo.toml").unwrap().packages[0]
        .targets
        .push(Target {
            name: "app".into(),
            kind: vec!["lib".into()],
            src_path: "app/lib.rs".into(),
        });
    assert!(super::super::check_registry(&input, "bindings.toml").is_empty());
    input
        .files
        .insert("bindings.toml".into(), "binding=[]".into());
    let diagnostics = super::super::check_registry(&input, "bindings.toml");
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].code, Code::DomainValue);
    assert!(diagnostics[0].message.contains("app/values.rs::Binding"));
    assert!(diagnostics[0].message.contains("needs exactly one"));
}

#[test]
fn whole_standard_values_resolve_after_glob_imports_without_ignoring_shadowing() {
    use super::{check_sources, roots, Files};
    for value in ["String", "bool", "u64", "i32", "Option<Vec<String>>"] {
        let binding =
            canonical("u64", "of").replace("type Value = Leaf;", &format!("type Value = {value};"));
        let registry = policy("domain").replace("app/lib.rs", "app/values.rs");
        let mut files = Files::from([
            ("app/lib.rs".into(), "mod values;".into()),
            ("app/values.rs".into(), format!("use super::*; {binding}")),
            ("bindings.toml".into(), registry),
        ]);
        assert_eq!(
            check_sources(&files, roots(), "bindings.toml"),
            Ok(()),
            "{value}"
        );
        files.insert(
            "app/lib.rs".into(),
            "mod values; struct String { hidden: EntityRef }".into(),
        );
        if value.contains("String") {
            let error = check_sources(&files, roots(), "bindings.toml")
                .unwrap_err()
                .join("\n");
            assert!(error.contains("EntityRef"), "{error}");
        }
        files.insert(
            "app/lib.rs".into(),
            "mod values; use unavailable::*;".into(),
        );
        assert!(check_sources(&files, roots(), "bindings.toml").is_err());
    }
}

#[test]
fn generic_arguments_follow_parent_globs_and_grouped_foreign_reexports() {
    use super::{check_sources, roots, Files};
    let binding =
        canonical("u64", "delegated").replace("type Value = Leaf;", "type Value = Amount<Unit>;");
    let registry = policy("domain").replace("app/lib.rs", "app/transcript/binding.rs");
    let mut files = Files::from([
        ("app/lib.rs".into(), "mod transcript;".into()),
        (
            "app/transcript.rs".into(),
            "pub use owner::model::{Amount, Unit}; mod binding;".into(),
        ),
        (
            "app/transcript/binding.rs".into(),
            format!("use super::*; {binding}"),
        ),
        ("owner/lib.rs".into(), "pub mod model;".into()),
        (
            "owner/model.rs".into(),
            "mod number; pub use number::{Amount, Unit};".into(),
        ),
        ("bindings.toml".into(), registry),
    ]);
    let mut roots = roots();
    roots.insert("owner".into(), "owner/lib.rs".into());
    for payload in ["", "{ hidden: EntityRef }"] {
        let unit = if payload.is_empty() { ";" } else { payload };
        files.insert("owner/model/number.rs".into(), format!(
            "use std::marker::PhantomData; pub struct Unit{unit} pub struct Amount<T> {{ minor: i64, unit: PhantomData<T> }}"
        ));
        let result = check_sources(&files, roots.clone(), "bindings.toml");
        if payload.is_empty() {
            assert_eq!(result, Ok(()));
        } else {
            let error = result.unwrap_err().join("\n");
            assert!(error.contains("EntityRef"), "{error}");
        }
    }
    // Distinct glob candidates must refuse rather than selecting one owner.
    files.insert(
        "app/transcript.rs".into(),
        "pub use owner::model::*; pub use other::*; mod binding; mod other { pub struct Unit; }"
            .into(),
    );
    let error = check_sources(&files, roots, "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("ambiguous glob"), "{error}");
}

#[test]
fn source_preserving_namespace_attributes_do_not_license_type_transformations() {
    use super::{check_sources, roots, Files};
    let binding = canonical("u64", "of");
    let registry = policy("domain").replace("app/lib.rs", "app/values.rs");
    let mut files = Files::from([
        (
            "app/lib.rs".into(),
            "#![cfg_attr(test, allow(unused))] #[macro_use] mod definitions; mod values;".into(),
        ),
        (
            "app/definitions.rs".into(),
            "#[macro_export] macro_rules! unrelated { () => {} }".into(),
        ),
        ("app/values.rs".into(), binding.clone()),
        ("bindings.toml".into(), registry),
    ]);
    assert_eq!(check_sources(&files, roots(), "bindings.toml"), Ok(()));
    for root in [
        "#[cfg_attr(all(), path=\"other.rs\")] mod values;",
        "#![cfg_attr(test, replace)] mod values;",
    ] {
        files.insert("app/lib.rs".into(), root.into());
        let error = check_sources(&files, roots(), "bindings.toml")
            .unwrap_err()
            .join("\n");
        assert!(error.contains("attribute"), "{error}");
    }
    files.insert(
        "app/lib.rs".into(),
        "#[macro_use] mod definitions; mod values;".into(),
    );
    files.insert("app/values.rs".into(), format!("unrelated!(); {binding}"));
    let error = check_sources(&files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("unknown binding macro origin"), "{error}");
}
