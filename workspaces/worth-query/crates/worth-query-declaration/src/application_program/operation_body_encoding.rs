//! Program projection of schema-owned executable values into canonical records.
use super::semantic_encoding::framed_record;
use crate::application_schema::ApplicationOperationBodyProgram;
mod base;
use base::operation_body_base_record;

#[cfg(test)]
mod children_tests;
mod enrollment;
#[cfg(test)]
mod reference_input_tests;
mod related_output;
mod related_source;
mod succession;
pub(super) fn operation_body_record(body: &ApplicationOperationBodyProgram) -> String {
    let record = related_output::operation_body_record(body);
    let record = if let Some(bound) = body
        .creation()
        .and_then(|creation| creation.subject_impact_bound())
    {
        framed_record(
            "operation-body-subject-impacts-v1",
            [
                ("body", record),
                ("maximum-subjects", bound.get().to_string()),
            ],
        )
    } else {
        record
    };
    let record = if body.values().is_empty() {
        record
    } else {
        framed_record(
            "operation-body-values-v1",
            std::iter::once(("body", record)).chain(
                body.values()
                    .iter()
                    .map(|value| ("value", value.canonical_record())),
            ),
        )
    };
    let record = if body.scope_fields().iter().any(|step| {
        matches!(
            step,
            crate::application_schema::ApplicationOperationBodyScopeField::ExpectInput { .. }
        )
    }) {
        framed_record("operation-body-input-expectations-v1", [("body", record)])
    } else {
        record
    };
    let record = if body.retirements().is_empty() {
        record
    } else {
        framed_record(
            "operation-body-owned-selection-retirements-v1",
            std::iter::once(("body", record)).chain(
                body.retirements()
                    .iter()
                    .map(|rule| ("retirement", rule.canonical_record())),
            ),
        )
    };
    let record = if body.path_input_expectations().is_empty() {
        record
    } else {
        framed_record(
            "operation-body-path-input-expectations-v1",
            std::iter::once(("body", record)).chain(
                body.path_input_expectations()
                    .iter()
                    .map(|guard| ("path-input", guard.canonical_record())),
            ),
        )
    };
    let record = if body.path_value_assignments().is_empty() {
        record
    } else {
        framed_record(
            "operation-body-path-value-assignments-v1",
            std::iter::once(("body", record)).chain(
                body.path_value_assignments()
                    .iter()
                    .map(|step| ("path-value", step.canonical_record())),
            ),
        )
    };
    let record = if body
        .creation()
        .is_some_and(|creation| creation.retires_predecessor())
    {
        framed_record("operation-body-retiring-replacement-v1", [("body", record)])
    } else {
        record
    };
    let record = if body
        .creation()
        .is_some_and(|creation| creation.reuses_current_if_unchanged())
    {
        framed_record(
            "operation-body-complete-content-reuse-v1",
            [("body", record)],
        )
    } else {
        record
    };
    let record = if body.reference_inputs().is_empty() {
        record
    } else {
        framed_record(
            "operation-body-reference-inputs-v1",
            std::iter::once(("body", record)).chain(
                body.reference_inputs()
                    .iter()
                    .map(|reference| ("reference-input", reference.canonical_record())),
            ),
        )
    };
    let Some(creation) = body.creation().filter(|c| !c.children().is_empty()) else {
        return record;
    };
    framed_record(
        "operation-body-child-sites-v1",
        std::iter::once(("body", record)).chain(creation.children().iter().map(|child| {
            (
                "child",
                framed_record(
                    "child-creation-v1",
                    [
                        ("parent", child.parent_key.clone()),
                        ("creation", base::creation_record(&child.creation)),
                        ("link", child.creation.links()[0].canonical_record()),
                    ],
                ),
            )
        })),
    )
}

fn operation_body_lifecycle_record(body: &ApplicationOperationBodyProgram) -> String {
    if let Some(lifecycle) = body
        .creation()
        .and_then(|creation| creation.signing_lifecycle())
    {
        return framed_record(
            "operation-body-v12",
            [
                ("body", operation_body_signing_record(body)),
                ("signing-lifecycle", lifecycle.canonical_record()),
            ],
        );
    }
    operation_body_signing_record(body)
}
fn operation_body_signing_record(body: &ApplicationOperationBodyProgram) -> String {
    let record = operation_body_link_record(body);
    if let Some(signing) = body.creation().and_then(|creation| creation.signing()) {
        return framed_record(
            "operation-body-v11",
            [("body", record), ("signing", signing.canonical_record())],
        );
    }
    record
}
fn operation_body_link_record(body: &ApplicationOperationBodyProgram) -> String {
    let record = operation_body_output_record(body);
    let Some(creation) = body
        .creation()
        .filter(|creation| !creation.links().is_empty())
    else {
        return record;
    };
    framed_record(
        if creation.links().iter().any(|link| link.posture == crate::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle) {
            "operation-body-v14"
        } else { "operation-body-v10" },
        [
            ("body", record),
            (
                "links",
                framed_record(
                    "creation-links-v1",
                    creation
                        .links()
                        .iter()
                        .map(|link| ("link", link.canonical_record())),
                ),
            ),
        ],
    )
}

fn operation_body_output_record(body: &ApplicationOperationBodyProgram) -> String {
    let record = operation_body_optional_record(body);
    if body.outputs().is_empty() {
        return record;
    }
    framed_record(
        "operation-body-v9",
        [
            ("body", record),
            (
                "outputs",
                framed_record(
                    "outputs-v1",
                    body.outputs()
                        .iter()
                        .map(|output| ("output", output.canonical_record())),
                ),
            ),
        ],
    )
}

fn operation_body_optional_record(body: &ApplicationOperationBodyProgram) -> String {
    let record = operation_body_scope_record(body);
    if body.has_optional_inputs() {
        framed_record("operation-body-v8", [("body", record)])
    } else {
        record
    }
}

fn operation_body_scope_record(body: &ApplicationOperationBodyProgram) -> String {
    let record = operation_body_requirements_record(body);
    if body.explicit_scope_entity().is_some() || body.is_creation_only() {
        return framed_record(
            "operation-body-v7",
            [
                (
                    "scope-entity",
                    body.scope_entity().unwrap_or_default().to_owned(),
                ),
                ("body", record),
            ],
        );
    }
    record
}

fn operation_body_requirements_record(body: &ApplicationOperationBodyProgram) -> String {
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
