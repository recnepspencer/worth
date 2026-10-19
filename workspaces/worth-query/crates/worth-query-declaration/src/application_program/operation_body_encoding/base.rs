//! Historical scalar/creation record encoding; optional extensions wrap it.
use super::framed_record;
use crate::application_schema::{
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram,
    ApplicationOperationBodyScopeField,
};
pub(super) fn operation_body_base_record(body: &ApplicationOperationBodyProgram) -> String {
    let has_requirement = body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
        )
    });
    let has_optional = body.scope_fields().iter().any(|step| {
        matches!(
            step,
            ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                | ApplicationOperationBodyScopeField::ClearOptional { .. }
        )
    });
    let steps = body.scope_fields().iter().enumerate().map(|(index, step)| {
        let (source_name, source) = match step {
            ApplicationOperationBodyScopeField::AssignValue { value, .. } => {
                ("computed-value", value.clone())
            }
            ApplicationOperationBodyScopeField::Expect { value, .. }
            | ApplicationOperationBodyScopeField::Assign { value, .. } => (
                "literal",
                worth_foundational::facade::prepare_aspect_value_identity_basis(value)
                    .as_str()
                    .to_owned(),
            ),
            ApplicationOperationBodyScopeField::ExpectInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::AssignInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::SetOptionalInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::AssignOptionalInput {
                member,
                scalar_family,
                ..
            } => (
                if step.input_is_optional() {
                    "optional-input"
                } else {
                    "input"
                },
                framed_record(
                    "input-member-v1",
                    [
                        ("member", member.to_owned()),
                        ("family", scalar_family.canonical_name().to_owned()),
                    ],
                ),
            ),
            ApplicationOperationBodyScopeField::ClearOptional { scalar_family, .. } => {
                ("clear", scalar_family.canonical_name().to_owned())
            }
            ApplicationOperationBodyScopeField::RequirePresent { scalar_family, .. } => {
                ("require-present", scalar_family.canonical_name().to_owned())
            }
        };
        framed_record(
            if has_requirement {
                "operation-body-step-v3"
            } else if has_optional {
                "operation-body-step-v2"
            } else {
                "operation-body-step-v1"
            },
            [
                ("index", index.to_string()),
                (
                    "kind",
                    match step {
                        ApplicationOperationBodyScopeField::Expect { .. } => "expect",
                        ApplicationOperationBodyScopeField::ExpectInput { .. } => "expect-input",
                        ApplicationOperationBodyScopeField::Assign { .. } => "assign",
                        ApplicationOperationBodyScopeField::AssignValue { .. } => "assign-value",
                        ApplicationOperationBodyScopeField::AssignInput { .. } => "assign-input",
                        ApplicationOperationBodyScopeField::SetOptionalInput { .. } => {
                            "set-optional-input"
                        }
                        ApplicationOperationBodyScopeField::AssignOptionalInput { .. } => {
                            "assign-optional-input"
                        }
                        ApplicationOperationBodyScopeField::ClearOptional { .. } => {
                            "clear-optional"
                        }
                        ApplicationOperationBodyScopeField::RequirePresent { .. } => {
                            "require-present"
                        }
                    }
                    .to_owned(),
                ),
                ("entity", step.field().entity().to_owned()),
                ("aspect", step.field().aspect().to_owned()),
                ("field", step.field().field().to_owned()),
                ("binding", step.binding().as_str().to_owned()),
                (source_name, source),
            ],
        )
    });
    let scope = framed_record(
        if has_requirement {
            "operation-body-v3"
        } else if has_optional {
            "operation-body-v2"
        } else {
            "operation-body-v1"
        },
        steps.map(|step| ("step", step)),
    );
    let Some(creation) = body.creation() else {
        return scope;
    };
    let has_scope_source = creation
        .fields()
        .iter()
        .any(|step| step.source_field().is_some());
    let created = creation_record(creation);
    framed_record(
        if has_requirement {
            "operation-body-v5"
        } else if has_optional {
            "operation-body-v4"
        } else if has_scope_source {
            "operation-body-v3"
        } else {
            "operation-body-v2"
        },
        [("scope", scope), ("creation", created)],
    )
}

pub(super) fn creation_record(
    creation: &crate::application_schema::ApplicationOperationBodyCreation,
) -> String {
    let has_scope_source = creation
        .fields()
        .iter()
        .any(|step| step.source_field().is_some());
    let has_principal_source = creation
        .fields()
        .iter()
        .any(|step| step.principal_source().is_some());
    let fields = creation.fields().iter().enumerate().map(|(index, step)| {
        let (source_name, source) = match step {
            ApplicationOperationBodyCreatedField::CapturedPathField {
                source,
                scalar_family,
                role,
                target_presence,
                ..
            } => (
                "classified-path-field",
                framed_record(
                    "classified-path-source-v1",
                    [
                        ("path", source.canonical_record()),
                        ("family", scalar_family.canonical_name().to_owned()),
                        ("role", role.canonical_name().to_owned()),
                        (
                            "target-presence",
                            target_presence.canonical_name().to_owned(),
                        ),
                    ],
                ),
            ),
            ApplicationOperationBodyCreatedField::Computed { value, .. } => {
                ("computed-value", value.clone())
            }
            ApplicationOperationBodyCreatedField::Literal { value, .. } => (
                "literal",
                worth_foundational::facade::prepare_aspect_value_identity_basis(value)
                    .as_str()
                    .to_owned(),
            ),
            ApplicationOperationBodyCreatedField::Input {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyCreatedField::OptionalInput {
                member,
                scalar_family,
                ..
            } => (
                if step.input_is_optional() {
                    "optional-input"
                } else {
                    "input"
                },
                framed_record(
                    "input-member-v1",
                    [
                        ("member", member.to_owned()),
                        ("family", scalar_family.canonical_name().to_owned()),
                    ],
                ),
            ),
            ApplicationOperationBodyCreatedField::ScopeField {
                source,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyCreatedField::PresentScopeField {
                source,
                scalar_family,
                ..
            } => (
                if matches!(
                    step,
                    ApplicationOperationBodyCreatedField::PresentScopeField { .. }
                ) {
                    "present-scope-field"
                } else {
                    "scope-field"
                },
                framed_record(
                    "scope-source-v1",
                    [
                        ("entity", source.entity().to_owned()),
                        ("aspect", source.aspect().to_owned()),
                        ("field", source.field().to_owned()),
                        ("family", scalar_family.canonical_name().to_owned()),
                    ],
                ),
            ),
            ApplicationOperationBodyCreatedField::CapturedScopeField {
                source,
                scalar_family,
                source_presence,
                role,
                ..
            } => (
                "classified-scope-field",
                framed_record(
                    "classified-scope-source-v1",
                    [
                        ("entity", source.entity().to_owned()),
                        ("aspect", source.aspect().to_owned()),
                        ("field", source.field().to_owned()),
                        ("family", scalar_family.canonical_name().to_owned()),
                        ("presence", source_presence.canonical_name().to_owned()),
                        ("role", role.canonical_name().to_owned()),
                    ],
                ),
            ),
            ApplicationOperationBodyCreatedField::PrincipalIdentity {
                source,
                scalar_family,
                mutation_binding,
                principal_binding,
                ..
            } => (
                "principal-identity",
                framed_record(
                    "principal-source-v1",
                    [
                        ("mutation-binding", mutation_binding.clone()),
                        ("principal-binding", principal_binding.clone()),
                        ("entity", source.entity().to_owned()),
                        ("aspect", source.aspect().to_owned()),
                        ("field", source.field().to_owned()),
                        ("family", scalar_family.canonical_name().to_owned()),
                    ],
                ),
            ),
            ApplicationOperationBodyCreatedField::RelatedField {
                source,
                scalar_family,
                ..
            } => (
                "related-field",
                framed_record(
                    "related-field-source-v1",
                    [
                        ("source", source.canonical_record()),
                        ("family", scalar_family.canonical_name().to_owned()),
                    ],
                ),
            ),
            ApplicationOperationBodyCreatedField::SigningProvenance { .. } => (
                "signing-provenance",
                "native-deferred-provenance-v1".to_owned(),
            ),
        };
        framed_record(
            if step.related_source().is_some() {
                "operation-body-created-field-v4"
            } else if has_principal_source {
                "operation-body-created-field-v3"
            } else if has_scope_source {
                "operation-body-created-field-v2"
            } else {
                "operation-body-created-field-v1"
            },
            [
                ("index", index.to_string()),
                ("entity", step.field().entity().to_owned()),
                ("aspect", step.field().aspect().to_owned()),
                ("field", step.field().field().to_owned()),
                ("binding", step.binding().as_str().to_owned()),
                (source_name, source),
            ],
        )
    });
    framed_record(
        if has_principal_source {
            "operation-body-creation-v3"
        } else if has_scope_source {
            "operation-body-creation-v2"
        } else {
            "operation-body-creation-v1"
        },
        [
            ("entity", creation.entity().to_owned()),
            ("key", creation.key().to_owned()),
            (
                "fields",
                framed_record("fields-v1", fields.map(|field| ("field", field))),
            ),
        ],
    )
}
