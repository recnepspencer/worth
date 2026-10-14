//! Program projection of schema-owned executable values into canonical records.
use super::semantic_encoding::framed_record;
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};

pub(super) fn operation_body_record(body: &ApplicationOperationBodyProgram) -> String {
    let steps = body.scope_fields().iter().enumerate().map(|(index, step)| {
        let (source_name, source) = match step.literal_value() {
            Some(value) => (
                "literal",
                worth_foundational::facade::prepare_aspect_value_identity_basis(value)
                    .as_str()
                    .to_owned(),
            ),
            None => (
                "input",
                framed_record(
                    "input-member-v1",
                    [
                        (
                            "member",
                            step.input_member().expect("input assignment").to_owned(),
                        ),
                        ("family", step.scalar_family().canonical_name().to_owned()),
                    ],
                ),
            ),
        };
        framed_record(
            "operation-body-step-v1",
            [
                ("index", index.to_string()),
                (
                    "kind",
                    match step {
                        ApplicationOperationBodyScopeField::Expect { .. } => "expect",
                        ApplicationOperationBodyScopeField::Assign { .. } => "assign",
                        ApplicationOperationBodyScopeField::AssignInput { .. } => "assign-input",
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
    framed_record("operation-body-v1", steps.map(|step| ("step", step)))
}
