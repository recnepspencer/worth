//! Program projection of schema-owned executable values into canonical records.
use super::semantic_encoding::framed_record;
use crate::application_schema::ApplicationOperationBodyProgram;
mod base;
use base::operation_body_base_record;

mod enrollment;
mod related_source;
mod succession;
pub(super) use succession::operation_body_record;

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
