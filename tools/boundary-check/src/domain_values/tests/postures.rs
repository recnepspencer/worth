//! Classification is mandatory; only reviewed diagnostics skip the field walk.
use super::{registration, run};

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
