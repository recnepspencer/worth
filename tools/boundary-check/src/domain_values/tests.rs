use super::*;
mod consumer;

fn roots() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("app".into(), "app/lib.rs".into()),
        ("worth_query_decl".into(), "platform/decl/lib.rs".into()),
    ])
}

fn run(source: &str, registrations: &str) -> Result<(), Vec<String>> {
    let files = Files::from([
        ("app/lib.rs".into(), source.into()),
        ("bindings.toml".into(), registrations.into()),
    ]);
    check_sources(&files, roots(), "bindings.toml")
}
fn registration(binding: &str) -> String {
    format!("[[binding]]\nsource = 'app/lib.rs'\nbinding = '{binding}'\nforbidden_types = ['GraphKey', 'ObjectName']\n")
}

#[test]
fn binary_and_canonical_claims_walk_all_owned_fields() {
    let source = r#"
        struct Wrapper<T> { value: T }
        struct Leaf { pair: (u64, u64), samples: Vec<Wrapper<Option<[u8; 4]>>> }
        ::worth_query_decl::facade::worth_query_value_binding!(Binary for Leaf { identity: "a.v1", binary: Codec, });
        ::worth_query_decl::facade::worth_query_value_binding!(Canonical for Leaf {
            identity: "b.v1", scalar: String, canonical: BOUNDS, domain_value,
        });
    "#;
    assert_eq!(
        run(
            source,
            &(registration("Binary") + &registration("Canonical"))
        ),
        Ok(())
    );
}

#[test]
fn an_unselected_enum_variant_cannot_hide_an_aliased_reference() {
    let source = r#"
        type Harmless = EntityRef;
        struct Wrapper<T> { value: T }
        enum Leaf { Ordinary(u64), Hidden(Wrapper<Harmless>) }
        ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: "a.v1", binary: Codec, });
    "#;
    let error = run(source, &registration("Binding"))
        .unwrap_err()
        .join("\n");
    assert!(error.contains("reference/key/name forbidden"), "{error}");
    assert!(error.contains("EntityRef"), "{error}");
}

#[test]
fn canonical_domain_bindings_receive_the_same_reference_guard() {
    let source = r#"
        struct GraphKey(u64);
        struct Leaf { value: Option<Box<GraphKey>> }
        ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf {
            identity: "a.v1", scalar: String, canonical: BOUNDS, domain_value,
        });
    "#;
    assert!(run(source, &registration("Binding"))
        .unwrap_err()
        .join("\n")
        .contains("GraphKey"));
}

#[test]
fn actual_imports_and_module_definitions_determine_the_referred_type() {
    let files = Files::from([
        ("app/lib.rs".into(), "mod owner; mod binding;".into()),
        ("app/owner.rs".into(), "pub struct ObjectName(u64); pub type Innocent = ObjectName;".into()),
        ("app/binding.rs".into(), r#"
            use crate::owner::Innocent as Alias;
            struct Wrap<T> { v: T }
            struct Leaf { v: Wrap<Alias> }
            ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: "a.v1", binary: Codec, });
        "#.into()),
        ("bindings.toml".into(), "[[binding]]\nsource='app/binding.rs'\nbinding='Binding'\nforbidden_types=['app::owner::ObjectName']".into()),
    ]);
    let error = check_sources(&files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(error.contains("app::owner::ObjectName"), "{error}");
}

#[test]
fn unresolved_imports_and_builtin_shadowing_fail_closed() {
    for declarations in [
        "use missing_owner::Opaque as Leaf;",
        "struct Vec { hidden: EntityRef } type Leaf = Vec;",
        "type Leaf = Unknown;",
        "struct Leaf { hidden: &'static u64 }",
    ] {
        let source = format!("{declarations} ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf {{ identity: \"a.v1\", binary: Codec, }});");
        assert!(
            run(&source, &registration("Binding")).is_err(),
            "{declarations}"
        );
    }
}

#[test]
fn a_manual_claim_uses_its_actual_associated_value() {
    let source = "struct Leaf { name: ObjectName } impl BindingTrait for Binding { type Value = Leaf; const DOMAIN_VALUE: bool = true; }";
    assert!(run(source, &registration("Binding"))
        .unwrap_err()
        .join("\n")
        .contains("ObjectName"));
    let corrupted = source.replace("type Value = Leaf", "type Value = EntityRef");
    assert!(run(&corrupted, &registration("Binding"))
        .unwrap_err()
        .join("\n")
        .contains("EntityRef"));
}

#[test]
fn unmarked_structured_canonical_artifacts_make_no_domain_atom_claim() {
    let source = "struct Leaf { id: EntityRef } ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: \"a.v1\", scalar: String, canonical: BOUNDS, });";
    assert_eq!(run(source, "binding=[]"), Ok(()));
}

#[test]
fn missing_stale_duplicate_and_forged_root_registrations_refuse() {
    let source = "struct Leaf(u64); ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: \"a.v1\", binary: Codec, });";
    assert!(run(source, "binding=[]")
        .unwrap_err()
        .join("\n")
        .contains("needs exactly one"));
    assert!(run(source, &registration("Other"))
        .unwrap_err()
        .join("\n")
        .contains("stale registration"));
    assert!(run(
        source,
        &(registration("Binding") + &registration("Binding"))
    )
    .unwrap_err()
    .join("\n")
    .contains("duplicate"));
    assert!(run(
        source,
        &(registration("Binding") + "value='DifferentRoot'\n")
    )
    .is_err());
}

#[test]
fn unknown_macro_generated_bindings_and_value_types_cannot_supply_a_proof() {
    let generated = r#"macro_rules! generate { () => { ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: "a.v1", binary: Codec, }); } }"#;
    assert!(run(generated, "binding=[]")
        .unwrap_err()
        .join("\n")
        .contains("unknown macro"));
    let opaque = "generate_type!(Leaf); ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: \"a.v1\", binary: Codec, });";
    assert!(run(opaque, &registration("Binding"))
        .unwrap_err()
        .join("\n")
        .contains("unknown binding macro origin generate_type"));
}

#[test]
fn module_paths_are_traced_from_the_actual_crate_source() {
    let files = Files::from([
        ("app/lib.rs".into(), "mod entry;".into()),
        ("app/entry.rs".into(), "#[path = \"values/mod.rs\"] mod values;".into()),
        ("app/values/mod.rs".into(), "mod binding;".into()),
        ("app/values/binding.rs".into(), "struct Leaf(u64); ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: \"a.v1\", binary: Codec, });".into()),
        ("bindings.toml".into(), "[[binding]]\nsource='app/values/binding.rs'\nbinding='Binding'\nforbidden_types=[]".into()),
    ]);
    assert_eq!(check_sources(&files, roots(), "bindings.toml"), Ok(()));
}

#[test]
fn public_internal_domain_arms_cannot_bypass_discovery() {
    let source = r#"struct Leaf(EntityRef);
        ::worth_query_decl::facade::worth_query_value_binding!(@canonical domain; Binding for Leaf {
            identity: "a.v1", scalar: String, canonical: BOUNDS,
        });"#;
    for registry in ["binding=[]".to_owned(), registration("Binding")] {
        assert!(run(source, &registry)
            .unwrap_err()
            .join("\n")
            .contains("internal domain declaration"));
    }
}

#[test]
fn absolute_extern_paths_and_imports_cannot_be_shadowed_by_a_local_module() {
    for value in ["::owner::Leaf", "Alias"] {
        let app = format!(
            r#"
            mod owner {{ pub struct Leaf(u64); }}
            use ::owner::Leaf as Alias;
            ::worth_query_decl::facade::worth_query_value_binding!(Binding for {value} {{ identity: "a.v1", binary: Codec, }});
        "#
        );
        let files = Files::from([
            ("app/lib.rs".into(), app),
            (
                "owner/lib.rs".into(),
                "pub struct Leaf { hidden: EntityRef }".into(),
            ),
            ("bindings.toml".into(), registration("Binding")),
        ]);
        let mut roots = BTreeMap::from([
            ("app".into(), "app/lib.rs".into()),
            ("owner".into(), "owner/lib.rs".into()),
        ]);
        roots.insert("worth_query_decl".into(), "platform/decl/lib.rs".into());
        let error = check_sources(&files, roots, "bindings.toml")
            .unwrap_err()
            .join("\n");
        assert!(error.contains("EntityRef"), "{error}");
    }
}

#[test]
fn transforming_module_and_definition_attributes_refuse_source_proofs() {
    for declaration in [
        "#[cfg_attr(all(), path=\"hidden.rs\")] mod chosen; type Leaf = chosen::Value;",
        "#[replace_fields] struct Leaf(u64);",
        "#[derive(UnknownTransform)] struct Leaf(u64);",
        "struct Leaf { #[cfg_attr(all(), replace)] value: u64 }",
    ] {
        let source = format!("{declaration} ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf {{ identity: \"a.v1\", binary: Codec, }});");
        let error = run(&source, &registration("Binding"))
            .unwrap_err()
            .join("\n");
        assert!(
            error.contains("attribute") || error.contains("derive"),
            "{error}"
        );
    }
}

#[test]
fn local_bindings_cannot_substitute_a_module_types_source_proof() {
    let source = r#"struct Leaf(u64); fn scope() {
        struct Leaf(EntityRef);
        ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: "a.v1", binary: Codec, });
    }"#;
    assert!(run(source, &registration("Binding"))
        .unwrap_err()
        .join("\n")
        .contains("local scope"));
}

#[test]
fn unknown_item_macros_cannot_shadow_a_prelude_leaf_type() {
    let source = r#"generate_imports!(); struct Leaf { v: Vec<u64> }
        ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: "a.v1", binary: Codec, });"#;
    assert!(run(source, &registration("Binding"))
        .unwrap_err()
        .join("\n")
        .contains("unknown binding macro origin"));
}

#[test]
fn generic_arguments_keep_their_callers_actual_module_provenance() {
    for payload in ["u32", "EntityRef"] {
        let files = Files::from([
            ("app/lib.rs".into(), r#"mod wrapping; mod values;
                ::worth_query_decl::facade::worth_query_value_binding!(Binding for wrapping::Wrap<values::Leaf> { identity: "a.v1", binary: Codec, });"#.into()),
            ("app/wrapping.rs".into(), "pub struct Wrap<T> { value: Option<T> }".into()),
            ("app/values.rs".into(), format!("pub struct Leaf {{ value: {payload} }}")),
            ("bindings.toml".into(), registration("Binding")),
        ]);
        let result = check_sources(&files, roots(), "bindings.toml");
        if payload == "u32" {
            assert_eq!(result, Ok(()));
        } else {
            assert!(result.unwrap_err().join("\n").contains("EntityRef"));
        }
    }
}

#[test]
fn resolved_self_recursion_is_closed_without_omitting_other_variants() {
    for payload in ["u32", "EntityRef"] {
        let source = format!(
            r#"enum Leaf {{ End({payload}), More(Box<Self>) }}
            ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf {{ identity: "a.v1", binary: Codec, }});"#
        );
        let result = run(&source, &registration("Binding"));
        if payload == "u32" {
            assert_eq!(result, Ok(()));
        } else {
            assert!(result.unwrap_err().join("\n").contains("EntityRef"));
        }
    }
}

#[test]
fn absolute_value_roots_still_require_untransformed_unique_associated_values() {
    for members in [
        "#[replace] type Value = ::std::string::String;",
        "#[cfg(any())] type Value = ::std::string::String; type Value = EntityRef;",
    ] {
        let source =
            format!("impl Trait for Binding {{ const DOMAIN_VALUE: bool = true; {members} }}");
        assert!(run(&source, &registration("Binding")).is_err());
    }
}

#[test]
fn helper_names_need_an_unshadowed_approved_derive_owner() {
    for declaration in [
        "use rogue::serde; #[serde] struct Leaf(u64);",
        "use rogue::serde; #[derive(::serde::Serialize)] #[serde] struct Leaf(u64);",
        "use rogue::replace as worth_value; #[worth_value] struct Leaf(u64);",
        "#[charged_bytes] struct Leaf(u64);",
        "#[serde] struct Leaf(u64);",
        "generate_imports!(); struct Leaf;",
    ] {
        let source = format!("{declaration} ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf {{ identity: \"a.v1\", binary: Codec, }});");
        assert!(run(&source, &registration("Binding")).is_err(), "{source}");
    }
    let source = r#"#[derive(::serde::Serialize)] struct Leaf { #[serde(rename = "v")] value: u64 }
        ::worth_query_decl::facade::worth_query_value_binding!(Binding for Leaf { identity: "a.v1", binary: Codec, });"#;
    assert_eq!(run(source, &registration("Binding")), Ok(()));
    let source = r#"generate_imports!(); impl Trait for Binding { const DOMAIN_VALUE: bool = true; type Value = (); }"#;
    assert!(run(source, &registration("Binding")).is_err());
}

#[test]
fn workspace_discovery_accepts_only_real_git_owned_cargo_manifests() {
    let mut files = Files::from([("bindings.toml".into(), "binding=[]".into())]);
    assert_eq!(workspace_manifests(&files, "bindings.toml"), Ok(Vec::new()));
    for registry in [
        "binding=[]\nworkspace_manifests=1",
        "binding=[]\nworkspace_manifests=[1]",
        "binding=[]\nworkspace_manifests=['outside/Cargo.toml']",
        "binding=[]\nworkspace_manifests=['app/lib.rs']",
    ] {
        files.insert("bindings.toml".into(), registry.into());
        assert!(workspace_manifests(&files, "bindings.toml").is_err());
    }
    files.insert("app/Cargo.toml".into(), "[workspace]".into());
    files.insert(
        "bindings.toml".into(),
        "binding=[]\nworkspace_manifests=['app/Cargo.toml']".into(),
    );
    assert_eq!(
        workspace_manifests(&files, "bindings.toml"),
        Ok(vec!["app/Cargo.toml".into()])
    );
}
