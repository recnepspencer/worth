//! Nested wire changes affect portable installed meaning, independent of Rust names.
use super::{canonical_identity, ApplicationSchemaCanonicalHeader};
use crate::application_schema::{
    ApplicationCanonicalValueContract, ApplicationFieldPresence, ApplicationSchemaMember,
};
use worth_foundational::facade::{CanonicalValue, ScalarAspectType, ValueDecodeBounds, WorthValue};

#[derive(WorthValue)]
struct Original {
    choice: Choice,
}
#[derive(WorthValue)]
struct RustRenamed {
    choice: Choice,
}
#[derive(WorthValue)]
enum Choice {
    First,
    Second(String),
}
#[derive(WorthValue)]
struct FieldAdded {
    choice: Choice,
    nondefault: bool,
}
#[derive(WorthValue)]
struct FieldRenamed {
    #[serde(rename = "selection")]
    choice: Choice,
}
#[derive(WorthValue)]
struct VariantAdded {
    choice: ExpandedChoice,
}
#[derive(WorthValue)]
enum ExpandedChoice {
    First,
    Second(String),
    Third(u64),
}

const BOUNDS: ValueDecodeBounds = ValueDecodeBounds::new(1024, 64, 16, 8);

fn identity<Value: CanonicalValue>(
    bounds: ValueDecodeBounds,
) -> crate::application_schema::ApplicationSchemaIdentity {
    let field = ApplicationSchemaMember::Field {
        entity: "Document".into(),
        aspect: "Facts".into(),
        field: "Saved".into(),
        presence: ApplicationFieldPresence::Required,
        scalar_family: ScalarAspectType::String,
        value_type: "example.saved.v1".into(),
        canonical: Some(ApplicationCanonicalValueContract::of::<Value>(bounds)),
        unit: None,
        frame: None,
        writable: true,
        equality_queryable: true,
        unique: false,
    };
    canonical_identity(
        ApplicationSchemaCanonicalHeader {
            owner: "example",
            name: "Documents",
            major: 1,
            minor: 0,
        },
        &[field],
        &[],
    )
}

#[test]
fn nested_field_enum_and_rename_changes_cannot_retain_the_original_schema_meaning() {
    let original = identity::<Original>(BOUNDS);
    assert_eq!(
        original,
        identity::<RustRenamed>(BOUNDS),
        "Rust type names are not portable wire meaning"
    );
    for changed in [
        identity::<FieldAdded>(BOUNDS),
        identity::<FieldRenamed>(BOUNDS),
        identity::<VariantAdded>(BOUNDS),
        identity::<Original>(ValueDecodeBounds::new(2048, 64, 16, 8)),
    ] {
        assert_ne!(original, changed);
    }
}

#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Recursive {
    Leaf,
    Branch { children: Vec<Recursive> },
}
#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecursiveRenamed {
    Leaf,
    Branch { children: Vec<RecursiveRenamed> },
}
#[derive(WorthValue)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecursiveChanged {
    Leaf,
    Branch {
        children: Vec<RecursiveChanged>,
        enabled: bool,
    },
}
#[test]
fn recursive_binders_are_finite_portable_and_include_nested_meaning() {
    assert_eq!(
        identity::<Recursive>(BOUNDS),
        identity::<RecursiveRenamed>(BOUNDS)
    );
    assert_ne!(
        identity::<Recursive>(BOUNDS),
        identity::<RecursiveChanged>(BOUNDS)
    );
    assert!(ApplicationCanonicalValueContract::of::<Recursive>(BOUNDS).supports_admission());
}
