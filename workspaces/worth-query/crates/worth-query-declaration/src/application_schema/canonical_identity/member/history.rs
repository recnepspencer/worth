use crate::application_history::ErasedApplicationHistoryDefinition;
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    definition: &ErasedApplicationHistoryDefinition,
) {
    let parts = definition.parts();
    basis.text(format!("{prefix}.kind"), "application-history");
    basis.text(format!("{prefix}.identity"), &parts.identity);
    basis.text(
        format!("{prefix}.history-type"),
        parts.history_type.as_str(),
    );
    for (field, value) in parts.access.canonical_fields() {
        basis.text(format!("{prefix}.access.{field}"), value);
    }
    basis.usize(format!("{prefix}.operation-count"), parts.operations.len());
    for (index, operation) in parts.operations.iter().enumerate() {
        basis.text(
            format!("{prefix}.operation[{index}].binding"),
            &operation.binding,
        );
        basis.text(
            format!("{prefix}.operation[{index}].label"),
            &operation.label,
        );
    }
}
