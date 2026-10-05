use super::WorthQueryGraphReadAccessRequirementRow;
use std::fmt::{self, Write};
use worth_foundational::facade::CanonicalDigestId;

pub(super) const DOMAIN: &str = "worth-query.application-query-access-requirements";
pub(super) const VERSION: &str = "worth-query-application-query-access-requirements-v1";

pub(super) enum RequirementDigestField<'a> {
    Digest(&'static str, CanonicalDigestId),
    Unsigned(&'static str, usize),
    Row(usize, &'a WorthQueryGraphReadAccessRequirementRow),
}

/// One field order and locus grammar serves ordinary and admitted owners.
/// Their sinks retain distinct resource and failure contracts.
pub(super) fn emit_fields<Stop>(
    read_graph: CanonicalDigestId,
    access_shape: CanonicalDigestId,
    selectivity_shape: CanonicalDigestId,
    rows: &[WorthQueryGraphReadAccessRequirementRow],
    mut emit: impl FnMut(RequirementDigestField<'_>) -> Result<(), Stop>,
) -> Result<(), Stop> {
    emit(RequirementDigestField::Digest("read-graph", read_graph))?;
    emit(RequirementDigestField::Digest("access-shape", access_shape))?;
    emit(RequirementDigestField::Digest(
        "selectivity-shape",
        selectivity_shape,
    ))?;
    emit(RequirementDigestField::Unsigned("row-count", rows.len()))?;
    for (index, row) in rows.iter().enumerate() {
        emit(RequirementDigestField::Row(index, row))?;
    }
    Ok(())
}

pub(super) fn write_row_locus(output: &mut dyn Write, index: usize) -> fmt::Result {
    write!(output, "row[{index}]")
}
