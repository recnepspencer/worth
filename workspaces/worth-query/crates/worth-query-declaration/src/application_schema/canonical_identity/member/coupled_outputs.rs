//! Empty coupling leaves every existing declaration identity unchanged.
use crate::{
    application_program::DeclaredCoupledOutputConnection,
    application_schema::canonical_basis::ApplicationSchemaCanonicalBasis,
};
pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    outputs: &[DeclaredCoupledOutputConnection],
) {
    if outputs.is_empty() {
        return;
    }
    basis.usize(format!("{prefix}.coupled-outputs.count"), outputs.len());
    for (index, output) in outputs.iter().enumerate() {
        let key = format!("{prefix}.coupled-outputs.{index}");
        for (field, value) in [
            ("connection", output.connection.as_str()),
            ("family", output.family.as_str()),
            ("source-binding", output.source_binding.as_str()),
            ("source-query", output.source_query.as_str()),
            ("source-input", output.source_input.as_str()),
        ] {
            basis.text(format!("{key}.{field}"), value);
        }
    }
}
