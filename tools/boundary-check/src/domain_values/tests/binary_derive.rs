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
