use serde::Serialize;
use worth_foundational::facade::AspectKey;

use crate::diagnostics::data::{
    aspect_shape_diagnostic_value, RelationalDiagnosticFields, RelationalDiagnosticValue as Value,
};
use crate::schema::data::aspect_merge_policy::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind,
};
use crate::schema::data::identity_declarations::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};

use super::{AspectBinding, AspectDeclarationTrace, AspectLoweringTrace};

pub(super) fn declaration_trace_diagnostic_fields(
    trace: &AspectDeclarationTrace,
) -> RelationalDiagnosticFields {
    let mut leaves = TypedLeaves::default();
    let trace = DeclarationTrace {
        kind_id: trace.kind_id.as_u64(),
        plan_revision: trace.plan_revision.0.to_string(),
        declarations: trace
            .declarations
            .iter()
            .map(|row| DeclarationRow {
                aspect_key: leaves.aspect_key(&row.aspect_key),
                binding: Binding::declaration(&row.binding, &mut leaves),
                contract_identity: row.contract_identity,
                contract_revision: row.contract_revision,
                aspect_shape: leaves.insert(aspect_shape_diagnostic_value(&row.aspect_shape)),
            })
            .collect(),
        identity_declarations: trace
            .identity_declarations
            .iter()
            .map(|declaration| IdentityDeclaration::new(declaration, &mut leaves))
            .collect(),
        merge_policy_declarations: trace
            .merge_policy_declarations
            .iter()
            .map(|declaration| MergePolicyDeclaration::new(declaration, &mut leaves))
            .collect(),
    };
    leaves.project(trace)
}

pub(super) fn lowering_trace_diagnostic_fields(
    trace: &AspectLoweringTrace,
) -> RelationalDiagnosticFields {
    let mut leaves = TypedLeaves::default();
    let trace = LoweringTrace {
        kind_id: trace.kind_id.as_u64(),
        plan_revision: trace.plan_revision.0.to_string(),
        bindings: trace
            .bindings
            .iter()
            .map(|row| LoweringRow {
                aspect_key: leaves.aspect_key(&row.aspect_key),
                target: Binding::lowered(&row.target, &mut leaves),
                aspect_shape: leaves.insert(aspect_shape_diagnostic_value(&row.aspect_shape)),
            })
            .collect(),
    };
    leaves.project(trace)
}

#[derive(Serialize)]
struct DeclarationTrace<'a> {
    kind_id: u64,
    plan_revision: String,
    declarations: Vec<DeclarationRow>,
    identity_declarations: Vec<IdentityDeclaration<'a>>,
    merge_policy_declarations: Vec<MergePolicyDeclaration<'a>>,
}

#[derive(Serialize)]
struct LoweringTrace {
    kind_id: u64,
    plan_revision: String,
    bindings: Vec<LoweringRow>,
}

#[derive(Serialize)]
struct DeclarationRow {
    aspect_key: TypedValue,
    binding: Binding,
    contract_identity: u64,
    contract_revision: u64,
    aspect_shape: TypedValue,
}

#[derive(Serialize)]
struct LoweringRow {
    aspect_key: TypedValue,
    target: Binding,
    aspect_shape: TypedValue,
}

#[derive(Serialize)]
struct Binding {
    #[serde(skip_serializing_if = "Option::is_none")]
    binding_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<TypedValue>,
}

impl Binding {
    fn declaration(binding: &AspectBinding, leaves: &mut TypedLeaves) -> Self {
        Self {
            binding_target: Some(Self::name(binding)),
            target_kind: None,
            field: Self::field(binding, leaves),
        }
    }

    fn lowered(binding: &AspectBinding, leaves: &mut TypedLeaves) -> Self {
        Self {
            binding_target: None,
            target_kind: Some(Self::name(binding)),
            field: Self::field(binding, leaves),
        }
    }

    fn name(binding: &AspectBinding) -> String {
        match binding {
            AspectBinding::EntityField { .. } => "entity_field".into(),
            AspectBinding::RelationField { .. } => "relation_field".into(),
            AspectBinding::RelationSourceEndpoint => "relation_source_endpoint".into(),
            AspectBinding::RelationTargetEndpoint => "relation_target_endpoint".into(),
            AspectBinding::LifecycleTransition => "lifecycle_transition".into(),
            _ => binding.canonical_name(),
        }
    }

    fn field(binding: &AspectBinding, leaves: &mut TypedLeaves) -> Option<TypedValue> {
        match binding {
            AspectBinding::EntityField { field } | AspectBinding::RelationField { field } => {
                Some(leaves.insert(Value::FieldKey(field.clone())))
            }
            _ => None,
        }
    }
}

#[derive(Serialize)]
struct IdentityDeclaration<'a> {
    scope: Scope,
    basis: Basis<'a>,
}

impl<'a> IdentityDeclaration<'a> {
    fn new(value: &'a IdentityBasisDeclaration, leaves: &mut TypedLeaves) -> Self {
        Self {
            scope: Scope::new(&value.scope, leaves),
            basis: Basis::new(&value.basis, leaves),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "scope_kind", rename_all = "snake_case")]
enum Scope {
    EntityKind { kind_id: u64 },
    RelationKind { kind_id: u64 },
    AspectKey { aspect_key: TypedValue },
}

impl Scope {
    fn new(scope: &IdentityBasisScope, leaves: &mut TypedLeaves) -> Self {
        match scope {
            IdentityBasisScope::EntityKind(kind_id) => Self::EntityKind {
                kind_id: kind_id.as_u64(),
            },
            IdentityBasisScope::RelationKind(kind_id) => Self::RelationKind {
                kind_id: kind_id.as_u64(),
            },
            IdentityBasisScope::AspectKey(key) => Self::AspectKey {
                aspect_key: leaves.aspect_key(key),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "basis_kind", rename_all = "snake_case")]
enum Basis<'a> {
    StorageIdentity,
    LineageIdentity,
    StructuralFingerprint,
    DeclaredKeySet {
        aspect_keys: Vec<TypedValue>,
    },
    Custom {
        name: &'a str,
        semantic_version: u32,
    },
}

impl<'a> Basis<'a> {
    fn new(basis: &'a IdentityBasisKind, leaves: &mut TypedLeaves) -> Self {
        match basis {
            IdentityBasisKind::StorageIdentity => Self::StorageIdentity,
            IdentityBasisKind::LineageIdentity => Self::LineageIdentity,
            IdentityBasisKind::StructuralFingerprint => Self::StructuralFingerprint,
            IdentityBasisKind::DeclaredKeySet(keys) => Self::DeclaredKeySet {
                aspect_keys: keys.iter().map(|key| leaves.aspect_key(key)).collect(),
            },
            IdentityBasisKind::Custom(identity) => Self::Custom {
                name: &identity.name,
                semantic_version: identity.semantic_version,
            },
        }
    }
}

#[derive(Serialize)]
struct MergePolicyDeclaration<'a> {
    aspect_key: TypedValue,
    policy: Policy<'a>,
}

impl<'a> MergePolicyDeclaration<'a> {
    fn new(value: &'a AspectMergePolicyDeclaration, leaves: &mut TypedLeaves) -> Self {
        Self {
            aspect_key: leaves.aspect_key(&value.aspect_key),
            policy: Policy::from(&value.policy),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "policy_kind", rename_all = "snake_case")]
enum Policy<'a> {
    FailOnConflict,
    LastWriterWins,
    MonotonicCounter,
    AdditiveSet,
    PreferRicher,
    Custom {
        name: &'a str,
        semantic_version: u32,
    },
}

impl<'a> From<&'a AspectMergePolicyKind> for Policy<'a> {
    fn from(policy: &'a AspectMergePolicyKind) -> Self {
        match policy {
            AspectMergePolicyKind::FailOnConflict => Self::FailOnConflict,
            AspectMergePolicyKind::RetiredWriterOrder => Self::LastWriterWins,
            AspectMergePolicyKind::MonotonicCounter => Self::MonotonicCounter,
            AspectMergePolicyKind::AdditiveSet => Self::AdditiveSet,
            AspectMergePolicyKind::RetiredValueShape => Self::PreferRicher,
            AspectMergePolicyKind::Custom(identity) => Self::Custom {
                name: &identity.name,
                semantic_version: identity.semantic_version,
            },
        }
    }
}

/// Typed leaves stay in custody while serde captures only the field structure.
#[derive(Serialize)]
struct TypedValue {
    typed_leaf: usize,
}

#[derive(Default)]
struct TypedLeaves(Vec<Value>);

impl TypedLeaves {
    fn insert(&mut self, value: Value) -> TypedValue {
        let typed_leaf = self.0.len();
        self.0.push(value);
        TypedValue { typed_leaf }
    }

    fn aspect_key(&mut self, key: &AspectKey) -> TypedValue {
        self.insert(Value::AspectKey(key.clone()))
    }

    fn project(mut self, trace: impl Serialize) -> RelationalDiagnosticFields {
        let tree = serde_json::to_value(trace).expect("trace DTO contains serializable values");
        project_fields(tree, &mut self.0).into()
    }
}

fn project_fields(tree: serde_json::Value, leaves: &mut [Value]) -> Value {
    match tree {
        serde_json::Value::Object(mut fields) => {
            if let Some(leaf) = fields.remove("typed_leaf") {
                let index = leaf.as_u64().expect("trace leaf index") as usize;
                return std::mem::replace(&mut leaves[index], Value::Null);
            }
            Value::object(
                fields
                    .into_iter()
                    .map(|(key, value)| (key, project_fields(value, leaves))),
            )
        }
        serde_json::Value::Array(rows) => {
            Value::array(rows.into_iter().map(|row| project_fields(row, leaves)))
        }
        serde_json::Value::String(text) => Value::String(text),
        serde_json::Value::Number(number) => {
            Value::Unsigned(number.as_u64().expect("unsigned trace field"))
        }
        _ => unreachable!("trace DTO serializes objects, arrays, text and unsigned integers"),
    }
}
