//! Program projection of schema-owned executable values into canonical records.
use super::semantic_encoding::framed_record;
use crate::application_schema::{
    ApplicationOperationBodyCreatedField, ApplicationOperationBodyProgram,
    ApplicationOperationBodyScopeField,
};

pub(super) fn operation_body_record(body: &ApplicationOperationBodyProgram) -> String {
    let base = operation_body_base_record(body);
    if body.requirements().is_empty() {
        return base;
    }
    framed_record(
        "operation-body-v6",
        [
            ("body", base),
            (
                "requirements",
                framed_record(
                    "requirements-v1",
                    body.requirements()
                        .iter()
                        .map(|requirement| ("requirement", requirement.canonical_record())),
                ),
            ),
        ],
    )
}

fn operation_body_base_record(body: &ApplicationOperationBodyProgram) -> String {
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
            ApplicationOperationBodyScopeField::Expect { value, .. }
            | ApplicationOperationBodyScopeField::Assign { value, .. } => (
                "literal",
                worth_foundational::facade::prepare_aspect_value_identity_basis(value)
                    .as_str()
                    .to_owned(),
            ),
            ApplicationOperationBodyScopeField::AssignInput {
                member,
                scalar_family,
                ..
            }
            | ApplicationOperationBodyScopeField::SetOptionalInput {
                member,
                scalar_family,
                ..
            } => (
                "input",
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
                        ApplicationOperationBodyScopeField::Assign { .. } => "assign",
                        ApplicationOperationBodyScopeField::AssignInput { .. } => "assign-input",
                        ApplicationOperationBodyScopeField::SetOptionalInput { .. } => {
                            "set-optional-input"
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
    let fields = creation.fields().iter().enumerate().map(|(index, step)| {
        let (source_name, source) = match step {
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
            } => (
                "input",
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
            } => (
                "scope-field",
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
        };
        framed_record(
            if has_scope_source {
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
    let created = framed_record(
        if has_scope_source {
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
    );
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
