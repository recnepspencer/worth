use serde::{Serialize, Serializer};

use super::terminal_projection::project_diagnostic_value_for_terminal_projection;
use super::RelationalDiagnosticFields;

pub(super) fn serialize_diagnostic_fields<S>(
    fields: &RelationalDiagnosticFields,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    project_diagnostic_value_for_terminal_projection(fields.root()).serialize(serializer)
}
