//! Exact installed catalogue dispatch of a recursive nondefault value.
use super::*;
use worth_foundational::facade::{CanonicalValue, ValueShapeBody, WorthValue};

#[derive(Debug, PartialEq, WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Tree {
    Leaf { text: String },
    Branch { children: Vec<Tree> },
}
worth_query_declaration::worth_query_value_binding! {
    TreeBinding for Tree { identity: "worth.tests.recursive.tree", scalar: String, canonical: BOUNDS }
}
schema!(
    TreeSchema,
    Node,
    Facts,
    Saved,
    Tree,
    TreeBinding,
    0x9174_0013
);

#[test]
fn recursive_installed_dispatch_has_exact_native_meaning_and_one_original_loan() {
    let installed = index::<TreeSchema>()
        .bind_application_schema(TreeSchema::declaration().unwrap())
        .unwrap();
    let binding = installed
        .value_bindings()
        .field("Node", "Facts", "Saved")
        .unwrap();
    let value = Tree::Branch {
        children: vec![Tree::Leaf {
            text: "nondefault nested".into(),
        }],
    };
    let raw = TreeBinding::encode(&value).unwrap();
    let literal = worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::from(
            r#"{"kind":"branch","children":[{"kind":"leaf","text":"nondefault nested"}]}"#,
        ),
    );
    assert_eq!(raw, literal);
    assert_eq!(
        binding
            .decode_admitted::<Tree>(&literal, &mut |_, _| Ok(()))
            .unwrap(),
        value
    );
    let mut denied = false;
    assert!(binding
        .decode_admitted::<Tree>(&literal, &mut |_, bytes| {
            assert!(!denied, "first refusal is latched across nested seeds");
            if bytes == "nondefault nested".len() {
                denied = true;
                return Err(ValueCodecDenial::Admission);
            }
            Ok(())
        })
        .is_err());
    assert!(denied);
    assert_eq!(
        binding
            .decode_admitted::<Tree>(&literal, &mut |_, _| Ok(()))
            .unwrap(),
        value
    );
    let meanings = installed.value_meaning().unwrap();
    let codec = meanings[0].fields()[0].codec_digest().unwrap();
    let worth_foundational::facade::AspectShape::Struct(shape) = installed
        .native_contracts()
        .aspect("Node", "Facts")
        .unwrap()
        .contract()
        .shape()
    else {
        panic!("native struct carrier");
    };
    assert_eq!(shape.fields()[0].codec(), Some(codec));
}

#[derive(serde::Serialize)]
struct Opaque(bool);
impl CanonicalValue for Opaque {
    const WIRE_NULLABLE: bool = false;
    const SHAPE_BODY: ValueShapeBody = { bool::SHAPE_BODY };
    fn validate_bounds(&self, _: ValueDecodeBounds) -> Result<(), ValueCodecDenial> {
        Ok(())
    }
    fn deserialize_bounded<'de, D: serde::Deserializer<'de>>(
        _: D,
        _: ValueDecodeBounds,
    ) -> Result<Self, D::Error> {
        panic!("unsupported owner must not enter its ordinary decoder")
    }
}
#[derive(WorthValue)]
enum Incomplete {
    Allowed,
    Hidden(Opaque),
}
worth_query_declaration::worth_query_value_binding! {
    IncompleteBinding for Incomplete { identity: "worth.tests.recursive.incomplete", scalar: String, canonical: BOUNDS }
}
schema!(
    IncompleteSchema,
    IncompleteNode,
    IncompleteFacts,
    IncompleteSaved,
    Incomplete,
    IncompleteBinding,
    0x9174_0014
);
#[test]
fn an_unselected_unadmitted_delegate_cannot_hide_inside_a_generated_seed() {
    let denial = index::<IncompleteSchema>()
        .bind_application_schema(IncompleteSchema::declaration().unwrap())
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledApplicationSchemaDenialKind::FieldBindingContractMismatch
    );
    let _ = Incomplete::Allowed;
}
