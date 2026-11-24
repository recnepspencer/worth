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
