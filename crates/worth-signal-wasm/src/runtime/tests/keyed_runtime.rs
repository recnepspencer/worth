use super::support::*;

#[test]
fn nested_family_scope_preconditions_reject_before_keyed_materialization() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source_family(KeyedSourceFamilySpec {
            family_id: "price".to_owned(),
            initial: SignalValue::Number(7.0),
            produces_aspects: None,
        })
        .unwrap();
    let mut family = KeyedRecipeFamilySpec {
        family_id: "child".to_owned(),
        reads: vec![RecipeFamilyReadSpec::Keyed {
            family_id: "price".to_owned(),
            scope: Some(
                serde_json::from_value(serde_json::json!({
                    "partitionFrom": "key", "detail": ""
                }))
                .unwrap(),
            ),
            aspects: crate::recipe::model::AspectSelectionSpec::default(),
        }],
        expr: read("price"),
        when: None,
        identity: None,
        produces_aspects: None,
    };
    assert_eq!(
        runtime
            .define_keyed_recipe_family(family.clone())
            .unwrap_err()
            .code,
        "invalidInput"
    );
    // Reusing the rejected id proves registration made no partial insertion.
    if let RecipeFamilyReadSpec::Keyed { scope, .. } = &mut family.reads[0] {
        *scope = Some(
            serde_json::from_value(serde_json::json!({
                "partitionFrom": "key", "detail": "spot"
            }))
            .unwrap(),
        );
    }
    runtime.define_keyed_recipe_family(family).unwrap();
    runtime
        .define_keyed_recipe_family(KeyedRecipeFamilySpec {
            family_id: "total".to_owned(),
            reads: ["price", "child"]
                .into_iter()
                .map(|family_id| RecipeFamilyReadSpec::Keyed {
                    family_id: family_id.to_owned(),
                    scope: None,
                    aspects: crate::recipe::model::AspectSelectionSpec::default(),
                })
                .collect(),
            expr: Expr::Sum {
                args: vec![read("price"), read("child")],
            },
            when: None,
            identity: None,
            produces_aspects: None,
        })
        .unwrap();
    assert_eq!(
        runtime.ensure_recipe_key("total", "").unwrap_err().code,
        "invalidInput"
    );
    for id in ["price::", "child::", "total::"] {
        assert_eq!(runtime.read_value(id).unwrap_err().code, "invalidInput");
    }
    assert_eq!(
        runtime.read_keyed_value("total", "cart-1").unwrap(),
        SignalValue::Number(14.0)
    );
    // Empty keys are still valid for unscoped source families.
    assert_eq!(
        runtime.read_keyed_value("price", "").unwrap(),
        SignalValue::Number(7.0)
    );
}

#[test]
fn keyed_families_expand_and_recompute() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source_family(KeyedSourceFamilySpec {
            family_id: "price".to_owned(),
            initial: SignalValue::Number(0.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_source_family(KeyedSourceFamilySpec {
            family_id: "tax".to_owned(),
            initial: SignalValue::Number(0.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_keyed_recipe_family(KeyedRecipeFamilySpec {
            family_id: "total".to_owned(),
            reads: vec![
                RecipeFamilyReadSpec::Keyed {
                    family_id: "price".to_owned(),
                    scope: None,
                    aspects: crate::recipe::model::AspectSelectionSpec::default(),
                },
                RecipeFamilyReadSpec::Keyed {
                    family_id: "tax".to_owned(),
                    scope: None,
                    aspects: crate::recipe::model::AspectSelectionSpec::default(),
                },
            ],
            expr: Expr::Sum {
                args: vec![read("price"), read("tax")],
            },
            when: None,
            identity: None,
            produces_aspects: None,
        })
        .unwrap();

    runtime
        .set_keyed_value("price", "cart-1", SignalValue::Number(100.0))
        .unwrap();
    runtime
        .set_keyed_value("tax", "cart-1", SignalValue::Number(8.0))
        .unwrap();

    let value = runtime.read_keyed_value("total", "cart-1").unwrap();
    assert_eq!(value, SignalValue::Number(108.0));
}

#[test]
fn keyed_families_can_mix_shared_and_keyed_reads() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "exposure".to_owned(),
            initial: SignalValue::Number(3.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_source_family(KeyedSourceFamilySpec {
            family_id: "pixelBase".to_owned(),
            initial: SignalValue::Number(0.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_keyed_recipe_family(KeyedRecipeFamilySpec {
            family_id: "pixel".to_owned(),
            reads: vec![
                RecipeFamilyReadSpec::Signal {
                    id: "exposure".to_owned(),
                    scope: None,
                    aspects: crate::recipe::model::AspectSelectionSpec::default(),
                },
                RecipeFamilyReadSpec::Keyed {
                    family_id: "pixelBase".to_owned(),
                    scope: None,
                    aspects: crate::recipe::model::AspectSelectionSpec::default(),
                },
            ],
            expr: Expr::Sum {
                args: vec![read("pixelBase"), read("exposure")],
            },
            when: None,
            identity: None,
            produces_aspects: None,
        })
        .unwrap();

    runtime
        .set_keyed_value("pixelBase", "10,5", SignalValue::Number(7.0))
        .unwrap();

    let value = runtime.read_keyed_value("pixel", "10,5").unwrap();
    assert_eq!(value, SignalValue::Number(10.0));
}
