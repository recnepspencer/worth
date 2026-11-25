use super::*;

#[test]
fn derived_worth_binary_value_registers_default_and_custom_bindings() {
    let source = r#"
        use worth_query_decl::facade::WorthBinaryValue;

        #[derive(Clone, Copy, Eq, PartialEq, WorthBinaryValue)]
        #[worth_binary(namespace = "app.window", version = 1)]
        pub struct Window { pub start: i64, pub end: i64 }

        #[derive(Clone, Copy, Eq, PartialEq, WorthBinaryValue)]
        #[worth_query(identity = "app.custom.v1", binding = CustomBoxBinding)]
        pub struct CustomBox(pub i64, pub i64);
    "#;
    assert_eq!(
        run(
            source,
            &(registration("WindowBinding") + &registration("CustomBoxBinding")),
        ),
        Ok(())
    );
}

#[test]
fn unregistered_derived_worth_binary_value_is_refused() {
    let source = r#"
        use worth_query_decl::facade::WorthBinaryValue;

        #[derive(Clone, Copy, WorthBinaryValue)]
        #[worth_binary(namespace = "app.window", version = 1)]
        pub struct Window { pub start: i64, pub end: i64 }
    "#;
    let error = run(source, "binding = []").unwrap_err().join("\n");
    assert!(error.contains("WindowBinding"), "{error}");
    assert!(error.contains("needs exactly one"), "{error}");
}

#[test]
fn derived_worth_binary_value_enforces_reference_closure_scrutiny() {
    let direct = r#"
        use worth_query_decl::facade::WorthBinaryValue;

        #[derive(Clone, Copy, WorthBinaryValue)]
        #[worth_binary(namespace = "app.bad", version = 1)]
        pub struct BadRef { pub id: EntityRef }
    "#;
    let err = run(direct, &registration("BadRefBinding"))
        .unwrap_err()
        .join("\n");
    assert!(err.contains("EntityRef"), "{err}");

    let aliased = r#"
        use worth_query_decl::facade::WorthBinaryValue;
        struct ObjectName(u64);
        type Hidden = ObjectName;

        #[derive(Clone, Copy, WorthBinaryValue)]
        #[worth_binary(namespace = "app.bad_alias", version = 1)]
        pub struct BadAlias { pub hidden: Hidden }
    "#;
    let err = run(aliased, &registration("BadAliasBinding"))
        .unwrap_err()
        .join("\n");
    assert!(err.contains("ObjectName"), "{err}");
}

#[test]
fn forged_origin_local_scope_and_macro_rules_derived_binary_values_refuse() {
    let forged = r#"
        use forged_crate::WorthBinaryValue;
        #[derive(WorthBinaryValue)]
        #[worth_binary(namespace = "app.forged", version = 1)]
        pub struct Forged { pub a: i64 }
    "#;
    let err = run(forged, &registration("ForgedBinding"))
        .unwrap_err()
        .join("\n");
    assert!(
        err.contains("unknown binding macro origin") || err.contains("unknown/shadowed derive"),
        "{err}"
    );

    let local = r#"
        use worth_query_decl::facade::WorthBinaryValue;
        fn inside() {
            #[derive(WorthBinaryValue)]
            #[worth_binary(namespace = "app.local", version = 1)]
            pub struct Local { pub a: i64 }
        }
    "#;
    let err = run(local, &registration("LocalBinding"))
        .unwrap_err()
        .join("\n");
    assert!(err.contains("local scope"), "{err}");

    let generated = r#"
        macro_rules! mk { () => {
            #[derive(WorthBinaryValue)]
            #[worth_binary(namespace = "app.mac", version = 1)]
            pub struct Mac { pub a: i64 }
        }; }
    "#;
    let err = run(generated, "binding = []").unwrap_err().join("\n");
    assert!(err.contains("unknown macro"), "{err}");
}

#[test]
fn imported_alias_worth_binary_value_requires_registration_and_closure_scrutiny() {
    let valid = r#"
        use worth_query_decl::facade::WorthBinaryValue as MyBin;

        #[derive(Clone, Copy, MyBin)]
        #[worth_binary(namespace = "app.aliased_ok", version = 1)]
        pub struct AliasedOk { pub a: i64, pub b: i64 }
    "#;
    assert_eq!(run(valid, &registration("AliasedOkBinding")), Ok(()));

    let unregistered = run(valid, "binding = []").unwrap_err().join("\n");
    assert!(unregistered.contains("AliasedOkBinding"), "{unregistered}");
    assert!(unregistered.contains("needs exactly one"), "{unregistered}");

    let bad_ref = r#"
        use worth_query_decl::facade::WorthBinaryValue as MyBin;

        #[derive(Clone, Copy, MyBin)]
        #[worth_binary(namespace = "app.aliased_bad", version = 1)]
        pub struct AliasedBad { pub id: EntityRef }
    "#;
    let err = run(bad_ref, &registration("AliasedBadBinding"))
        .unwrap_err()
        .join("\n");
    assert!(err.contains("EntityRef"), "{err}");

    let forged = r#"
        use forged_crate::WorthBinaryValue as MyBin;

        #[derive(Clone, Copy, MyBin)]
        #[worth_binary(namespace = "app.aliased_forged", version = 1)]
        pub struct AliasedForged { pub a: i64 }
    "#;
    let err = run(forged, &registration("AliasedForgedBinding"))
        .unwrap_err()
        .join("\n");
    assert!(
        err.contains("unknown binding macro origin") || err.contains("unknown/shadowed derive"),
        "{err}"
    );
}

#[test]
fn cfg_attr_derived_worth_binary_value_is_refused() {
    let direct_cfg = r#"
        use worth_query_decl::facade::WorthBinaryValue;

        #[cfg_attr(all(), derive(WorthBinaryValue))]
        #[worth_binary(namespace = "app.cfg_direct", version = 1)]
        pub struct CfgDirect { pub id: EntityRef }
    "#;
    let err = run(direct_cfg, "binding = []").unwrap_err().join("\n");
    assert!(err.contains("cfg_attr"), "{err}");

    let aliased_cfg = r#"
        use worth_query_decl::facade::WorthBinaryValue as MyBin;

        #[cfg_attr(all(), derive(MyBin))]
        #[worth_query(identity = "app.cfg_alias.v1")]
        pub struct CfgAlias { pub id: EntityRef }
    "#;
    let err = run(aliased_cfg, "binding = []").unwrap_err().join("\n");
    assert!(err.contains("cfg_attr"), "{err}");

    let nested_cfg = r#"
        use worth_query_decl::facade::WorthBinaryValue;

        #[cfg_attr(all(), cfg_attr(all(), derive(WorthBinaryValue)))]
        #[worth_binary(namespace = "app.cfg_nested", version = 1)]
        pub struct CfgNested { pub a: i64 }
    "#;
    let err = run(nested_cfg, &registration("CfgNestedBinding"))
        .unwrap_err()
        .join("\n");
    assert!(err.contains("cfg_attr"), "{err}");
}

#[test]
fn inline_and_out_of_line_child_super_aliases_resolve_through_module_tree() {
    // (a) Registered inline child module using `use super::Alias` is accepted when reference-free,
    //     and still enforces reference closure when carrying `EntityRef`.
    let inline_ok = r#"
        use worth_query_decl::facade::WorthBinaryValue as Alias;
        mod child {
            use super::Alias;
            #[derive(Clone, Copy, Alias)]
            #[worth_binary(namespace = "app.child_ok", version = 1)]
            pub struct ChildOk { pub a: i64 }
        }
    "#;
    assert_eq!(
        run(inline_ok, &registration("child::ChildOkBinding")),
        Ok(())
    );

    let inline_bad = r#"
        use worth_query_decl::facade::WorthBinaryValue as Alias;
        mod child {
            use super::Alias;
            #[derive(Clone, Copy, Alias)]
            #[worth_binary(namespace = "app.child_bad", version = 1)]
            pub struct ChildBad { pub id: EntityRef }
        }
    "#;
    let err = run(inline_bad, &registration("child::ChildBadBinding"))
        .unwrap_err()
        .join("\n");
    assert!(err.contains("EntityRef"), "{err}");

    // (b) Unregistered out-of-line child module file using `use super::Alias` + `worth_query(identity = ...)`
    //     is caught and refused (both unconditional and under `cfg_attr`).
    let out_of_line_unreg = Files::from([
        (
            "app/lib.rs".into(),
            "use worth_query_decl::facade::WorthBinaryValue as Alias;\nmod child;\n".into(),
        ),
        (
            "app/child.rs".into(),
            r#"
                use super::Alias;
                #[derive(Clone, Copy, Alias)]
                #[worth_query(identity = "app.child_sneaky.v1")]
                pub struct ChildSneaky { pub id: EntityRef }
            "#
            .into(),
        ),
        ("bindings.toml".into(), "binding = []".into()),
    ]);
    let err = check_sources(&out_of_line_unreg, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(err.contains("ChildSneakyBinding"), "{err}");
    assert!(err.contains("needs exactly one"), "{err}");

    let out_of_line_cfg = Files::from([
        (
            "app/lib.rs".into(),
            "use worth_query_decl::facade::WorthBinaryValue as Alias;\nmod child;\n".into(),
        ),
        (
            "app/child.rs".into(),
            r#"
                use super::Alias;
                #[cfg_attr(all(), derive(Alias))]
                #[worth_query(identity = "app.child_cfg.v1")]
                pub struct ChildCfg { pub id: EntityRef }
            "#
            .into(),
        ),
        ("bindings.toml".into(), "binding = []".into()),
    ]);
    let err = check_sources(&out_of_line_cfg, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(err.contains("cfg_attr"), "{err}");

    // (c) Registered out-of-line child module file with `use super::Alias` is accepted when clean,
    //     and unresolvable `use super::MissingAlias` fails closed.
    let out_of_line_ok = Files::from([
        (
            "app/lib.rs".into(),
            "use worth_query_decl::facade::WorthBinaryValue as Alias;\nmod child;\n".into(),
        ),
        (
            "app/child.rs".into(),
            r#"
                use super::Alias;
                #[derive(Clone, Copy, Alias)]
                #[worth_query(identity = "app.child_ok.v1")]
                pub struct ChildFileOk { pub a: i64 }
            "#
            .into(),
        ),
        (
            "bindings.toml".into(),
            "[[binding]]\nsource = 'app/child.rs'\nbinding = 'ChildFileOkBinding'\nposture='domain'\nreason='reference-free fixture value'\nforbidden_types = ['GraphKey', 'ObjectName']\n".into(),
        ),
    ]);
    assert_eq!(
        check_sources(&out_of_line_ok, roots(), "bindings.toml"),
        Ok(())
    );

    let unresolvable_super = Files::from([
        ("app/lib.rs".into(), "mod child;\n".into()),
        (
            "app/child.rs".into(),
            r#"
                use super::MissingAlias;
                #[derive(MissingAlias)]
                #[worth_query(identity = "app.child_missing.v1")]
                pub struct ChildMissing { pub a: i64 }
            "#
            .into(),
        ),
        ("bindings.toml".into(), "binding = []".into()),
    ]);
    let err = check_sources(&unresolvable_super, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(err.contains("unresolvable derive macro alias"), "{err}");
}

#[test]
fn alias_renamed_to_known_derive_name_still_resolves_through_module_tree() {
    let lib = "use worth_query_decl::facade::WorthBinaryValue as Alias;\nmod child;\n";
    let child_src = r#"
        use super::Alias as WorthQuery;
        #[derive(Clone, Copy, WorthQuery)]
        #[worth_query(identity = "app.disguised.v1")]
        pub struct Disguised { pub a: i64 }
    "#;
    let reg = "[[binding]]\nsource = 'app/child.rs'\nbinding = 'DisguisedBinding'\nposture='domain'\nreason='reference-free fixture value'\nforbidden_types = ['GraphKey', 'ObjectName']\n";

    // Registered clean struct using `use super::Alias as WorthQuery` is accepted.
    let ok_files = Files::from([
        ("app/lib.rs".into(), lib.into()),
        ("app/child.rs".into(), child_src.into()),
        ("bindings.toml".into(), reg.into()),
    ]);
    assert_eq!(check_sources(&ok_files, roots(), "bindings.toml"), Ok(()));

    // Unregistered struct using `use super::Alias as WorthQuery` is refused.
    let unreg_files = Files::from([
        ("app/lib.rs".into(), lib.into()),
        ("app/child.rs".into(), child_src.into()),
        ("bindings.toml".into(), "binding = []".into()),
    ]);
    let err = check_sources(&unreg_files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(err.contains("DisguisedBinding"), "{err}");
    assert!(err.contains("needs exactly one"), "{err}");

    // `cfg_attr` with `use super::Alias as WorthQuery` is refused.
    let cfg_files = Files::from([
        ("app/lib.rs".into(), lib.into()),
        (
            "app/child.rs".into(),
            r#"
                use super::Alias as WorthQuery;
                #[cfg_attr(all(), derive(WorthQuery))]
                #[worth_query(identity = "app.disguised_cfg.v1")]
                pub struct DisguisedCfg { pub id: EntityRef }
            "#
            .into(),
        ),
        ("bindings.toml".into(), "binding = []".into()),
    ]);
    let err = check_sources(&cfg_files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(err.contains("cfg_attr"), "{err}");

    // Even renaming to a builtin derive name (`Clone`) cannot bypass discovery.
    let builtin_alias_files = Files::from([
        ("app/lib.rs".into(), lib.into()),
        (
            "app/child.rs".into(),
            r#"
                use super::Alias as Clone;
                #[derive(Clone)]
                #[worth_query(identity = "app.disguised_clone.v1")]
                pub struct DisguisedClone { pub id: EntityRef }
            "#
            .into(),
        ),
        ("bindings.toml".into(), "binding = []".into()),
    ]);
    let err = check_sources(&builtin_alias_files, roots(), "bindings.toml")
        .unwrap_err()
        .join("\n");
    assert!(err.contains("DisguisedCloneBinding"), "{err}");
}
