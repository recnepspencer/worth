use crate::diagnostics::data::{
    aspect_shape_diagnostic_value, RelationalDiagnosticFields, RelationalDiagnosticValue,
};
use crate::identity::data::KindId;
use crate::schema::data::aspect_merge_policy::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind,
};
use crate::schema::data::identity_declarations::{
    IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};

use super::{
    AspectBinding, AspectContractPlanRevision, AspectDeclarationTrace, AspectDeclarationTraceRow,
    AspectLoweringTrace, AspectLoweringTraceRow,
};

pub(super) fn declaration_trace_diagnostic_fields(
    trace: &AspectDeclarationTrace,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        ("kind_id", kind_id_value(trace.kind_id)),
        ("plan_revision", plan_revision_value(trace.plan_revision)),
        (
            "declarations",
            RelationalDiagnosticValue::array(
                trace
                    .declarations
                    .iter()
                    .map(aspect_declaration_trace_row_value),
            ),
        ),
        (
            "identity_declarations",
            RelationalDiagnosticValue::array(
                trace
                    .identity_declarations
                    .iter()
                    .map(identity_basis_declaration_value),
            ),
        ),
        (
            "merge_policy_declarations",
            RelationalDiagnosticValue::array(
                trace
                    .merge_policy_declarations
                    .iter()
                    .map(aspect_merge_policy_declaration_value),
            ),
        ),
    ])
    .into()
}

pub(super) fn lowering_trace_diagnostic_fields(
    trace: &AspectLoweringTrace,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        ("kind_id", kind_id_value(trace.kind_id)),
        ("plan_revision", plan_revision_value(trace.plan_revision)),
        (
            "bindings",
            RelationalDiagnosticValue::array(
                trace.bindings.iter().map(aspect_lowering_trace_row_value),
            ),
        ),
    ])
    .into()
}

fn aspect_declaration_trace_row_value(
    row: &AspectDeclarationTraceRow,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(row.aspect_key.clone()),
        ),
        ("binding", aspect_binding_value(&row.binding)),
        (
            "contract_identity",
            RelationalDiagnosticValue::Unsigned(row.contract_identity),
        ),
        (
            "contract_revision",
            RelationalDiagnosticValue::Unsigned(row.contract_revision),
        ),
        (
            "aspect_shape",
            aspect_shape_diagnostic_value(&row.aspect_shape),
        ),
    ])
}

fn aspect_lowering_trace_row_value(row: &AspectLoweringTraceRow) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(row.aspect_key.clone()),
        ),
        ("target", aspect_binding_target_value(&row.target)),
        (
            "aspect_shape",
            aspect_shape_diagnostic_value(&row.aspect_shape),
        ),
    ])
}

fn aspect_binding_value(binding: &AspectBinding) -> RelationalDiagnosticValue {
    match binding {
        AspectBinding::EntityField { field } => RelationalDiagnosticValue::object([
            (
                "binding_target",
                RelationalDiagnosticValue::string("entity_field"),
            ),
            ("field", RelationalDiagnosticValue::FieldKey(field.clone())),
        ]),
        AspectBinding::RelationField { field } => RelationalDiagnosticValue::object([
            (
                "binding_target",
                RelationalDiagnosticValue::string("relation_field"),
            ),
            ("field", RelationalDiagnosticValue::FieldKey(field.clone())),
        ]),
        AspectBinding::RelationSourceEndpoint => RelationalDiagnosticValue::object([(
            "binding_target",
            RelationalDiagnosticValue::string("relation_source_endpoint"),
        )]),
        AspectBinding::RelationTargetEndpoint => RelationalDiagnosticValue::object([(
            "binding_target",
            RelationalDiagnosticValue::string("relation_target_endpoint"),
        )]),
        AspectBinding::LifecycleTransition => RelationalDiagnosticValue::object([(
            "binding_target",
            RelationalDiagnosticValue::string("lifecycle_transition"),
        )]),
        _ => RelationalDiagnosticValue::object([(
            "binding_target",
            RelationalDiagnosticValue::string(binding.canonical_name()),
        )]),
    }
}

fn aspect_binding_target_value(target: &AspectBinding) -> RelationalDiagnosticValue {
    match target {
        AspectBinding::EntityField { field } => RelationalDiagnosticValue::object([
            (
                "target_kind",
                RelationalDiagnosticValue::string("entity_field"),
            ),
            ("field", RelationalDiagnosticValue::FieldKey(field.clone())),
        ]),
        AspectBinding::RelationField { field } => RelationalDiagnosticValue::object([
            (
                "target_kind",
                RelationalDiagnosticValue::string("relation_field"),
            ),
            ("field", RelationalDiagnosticValue::FieldKey(field.clone())),
        ]),
        AspectBinding::RelationSourceEndpoint => RelationalDiagnosticValue::object([(
            "target_kind",
            RelationalDiagnosticValue::string("relation_source_endpoint"),
        )]),
        AspectBinding::RelationTargetEndpoint => RelationalDiagnosticValue::object([(
            "target_kind",
            RelationalDiagnosticValue::string("relation_target_endpoint"),
        )]),
        AspectBinding::LifecycleTransition => RelationalDiagnosticValue::object([(
            "target_kind",
            RelationalDiagnosticValue::string("lifecycle_transition"),
        )]),
        _ => RelationalDiagnosticValue::object([(
            "target_kind",
            RelationalDiagnosticValue::string(target.canonical_name()),
        )]),
    }
}

fn identity_basis_declaration_value(
    declaration: &IdentityBasisDeclaration,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        ("scope", identity_basis_scope_value(&declaration.scope)),
        ("basis", identity_basis_kind_value(&declaration.basis)),
    ])
}

fn identity_basis_scope_value(scope: &IdentityBasisScope) -> RelationalDiagnosticValue {
    match scope {
        IdentityBasisScope::EntityKind(kind_id) => RelationalDiagnosticValue::object([
            (
                "scope_kind",
                RelationalDiagnosticValue::string("entity_kind"),
            ),
            ("kind_id", kind_id_value(*kind_id)),
        ]),
        IdentityBasisScope::RelationKind(kind_id) => RelationalDiagnosticValue::object([
            (
                "scope_kind",
                RelationalDiagnosticValue::string("relation_kind"),
            ),
            ("kind_id", kind_id_value(*kind_id)),
        ]),
        IdentityBasisScope::AspectKey(aspect_key) => RelationalDiagnosticValue::object([
            (
                "scope_kind",
                RelationalDiagnosticValue::string("aspect_key"),
            ),
            (
                "aspect_key",
                RelationalDiagnosticValue::AspectKey(aspect_key.clone()),
            ),
        ]),
    }
}

fn identity_basis_kind_value(basis: &IdentityBasisKind) -> RelationalDiagnosticValue {
    match basis {
        IdentityBasisKind::StorageIdentity => RelationalDiagnosticValue::object([(
            "basis_kind",
            RelationalDiagnosticValue::string("storage_identity"),
        )]),
        IdentityBasisKind::LineageIdentity => RelationalDiagnosticValue::object([(
            "basis_kind",
            RelationalDiagnosticValue::string("lineage_identity"),
        )]),
        IdentityBasisKind::StructuralFingerprint => RelationalDiagnosticValue::object([(
            "basis_kind",
            RelationalDiagnosticValue::string("structural_fingerprint"),
        )]),
        IdentityBasisKind::DeclaredKeySet(aspect_keys) => RelationalDiagnosticValue::object([
            (
                "basis_kind",
                RelationalDiagnosticValue::string("declared_key_set"),
            ),
            (
                "aspect_keys",
                RelationalDiagnosticValue::array(
                    aspect_keys
                        .iter()
                        .cloned()
                        .map(RelationalDiagnosticValue::AspectKey),
                ),
            ),
        ]),
        IdentityBasisKind::Custom(identity) => RelationalDiagnosticValue::object([
            ("basis_kind", RelationalDiagnosticValue::string("custom")),
            (
                "name",
                RelationalDiagnosticValue::string(identity.name.as_ref()),
            ),
            (
                "semantic_version",
                RelationalDiagnosticValue::Unsigned(identity.semantic_version as u64),
            ),
        ]),
    }
}

fn aspect_merge_policy_declaration_value(
    declaration: &AspectMergePolicyDeclaration,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(declaration.aspect_key.clone()),
        ),
        (
            "policy",
            aspect_merge_policy_kind_value(&declaration.policy),
        ),
    ])
}

fn aspect_merge_policy_kind_value(policy: &AspectMergePolicyKind) -> RelationalDiagnosticValue {
    match policy {
        AspectMergePolicyKind::FailOnConflict => RelationalDiagnosticValue::object([(
            "policy_kind",
            RelationalDiagnosticValue::string("fail_on_conflict"),
        )]),
        AspectMergePolicyKind::RetiredWriterOrder => RelationalDiagnosticValue::object([(
            "policy_kind",
            RelationalDiagnosticValue::string("last_writer_wins"),
        )]),
        AspectMergePolicyKind::MonotonicCounter => RelationalDiagnosticValue::object([(
            "policy_kind",
            RelationalDiagnosticValue::string("monotonic_counter"),
        )]),
        AspectMergePolicyKind::AdditiveSet => RelationalDiagnosticValue::object([(
            "policy_kind",
            RelationalDiagnosticValue::string("additive_set"),
        )]),
        AspectMergePolicyKind::RetiredValueShape => RelationalDiagnosticValue::object([(
            "policy_kind",
            RelationalDiagnosticValue::string("prefer_richer"),
        )]),
        AspectMergePolicyKind::Custom(identity) => RelationalDiagnosticValue::object([
            ("policy_kind", RelationalDiagnosticValue::string("custom")),
            (
                "name",
                RelationalDiagnosticValue::string(identity.name.as_ref()),
            ),
            (
                "semantic_version",
                RelationalDiagnosticValue::Unsigned(identity.semantic_version as u64),
            ),
        ]),
    }
}

fn kind_id_value(kind_id: KindId) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::Unsigned(kind_id.as_u64())
}

fn plan_revision_value(plan_revision: AspectContractPlanRevision) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::string(plan_revision.0.to_string())
}
