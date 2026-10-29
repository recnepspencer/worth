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
    let mut stop = None;
    let mut step = |field: RequirementDigestField<'_>| -> bool {
        match emit(field) {
            Ok(()) => true,
            Err(error) => {
                stop = Some(error);
                false
            }
        }
    };
    if drive_fields(read_graph, access_shape, selectivity_shape, rows, &mut step) {
        Ok(())
    } else {
        Err(stop.expect("a refused requirement field stores its stop"))
    }
}

#[inline(never)]
fn drive_fields(
    read_graph: CanonicalDigestId,
    access_shape: CanonicalDigestId,
    selectivity_shape: CanonicalDigestId,
    rows: &[WorthQueryGraphReadAccessRequirementRow],
    emit: &mut dyn FnMut(RequirementDigestField<'_>) -> bool,
) -> bool {
    if !emit(RequirementDigestField::Digest("read-graph", read_graph)) {
        return false;
    }
    if !emit(RequirementDigestField::Digest("access-shape", access_shape)) {
        return false;
    }
    if !emit(RequirementDigestField::Digest(
        "selectivity-shape",
        selectivity_shape,
    )) {
        return false;
    }
    if !emit(RequirementDigestField::Unsigned("row-count", rows.len())) {
        return false;
    }
    for (index, row) in rows.iter().enumerate() {
        if !emit(RequirementDigestField::Row(index, row)) {
            return false;
        }
    }
    true
}

pub(super) fn write_row_locus(output: &mut dyn Write, index: usize) -> fmt::Result {
    write!(output, "row[{index}]")
}
