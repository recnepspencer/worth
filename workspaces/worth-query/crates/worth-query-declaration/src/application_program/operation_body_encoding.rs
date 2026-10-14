//! Program projection of schema-owned executable values into canonical records.
use super::semantic_encoding::framed_record;
use crate::application_schema::{
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};

pub(super) fn operation_body_record(body: &ApplicationOperationBodyProgram) -> String {
    let steps = body.scope_fields().iter().enumerate().map(|(index, step)| {
        framed_record(
            "operation-body-step-v1",
            [
                ("index", index.to_string()),
                (
                    "kind",
                    match step {
                        ApplicationOperationBodyScopeField::Expect { .. } => "expect",
                        ApplicationOperationBodyScopeField::Assign { .. } => "assign",
                    }
                    .to_owned(),
                ),
                ("entity", step.field().entity().to_owned()),
                ("aspect", step.field().aspect().to_owned()),
                ("field", step.field().field().to_owned()),
                ("binding", step.binding().as_str().to_owned()),
                (
                    "literal",
                    worth_foundational::facade::prepare_aspect_value_identity_basis(step.value())
                        .as_str()
                        .to_owned(),
                ),
            ],
        )
    });
    framed_record("operation-body-v1", steps.map(|step| ("step", step)))
}
