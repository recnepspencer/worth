//! Fresh readmission cannot discard peer authority or alias creation-site identity.
pub use super::program;
pub use worth_query_decl::facade::{application_query::*, application_schema::*};
pub fn replace_root(
    body: &ApplicationOperationBodyProgram,
    root: ApplicationOperationBodyCreation,
) -> ApplicationOperationBodyProgram {
    ApplicationOperationBodyProgram::from_untrusted_in_scope(
        "Row".into(),
        body.scope_fields().to_vec(),
        Some(root),
        body.requirements().to_vec(),
    )
    .with_untrusted_values(body.values().to_vec())
    .with_untrusted_path_value_assignments(body.path_value_assignments().to_vec())
    .with_untrusted_outputs(body.outputs().to_vec())
}
