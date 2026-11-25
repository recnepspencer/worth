//! Where-filter identity material. Absent filters contribute no entries, so
//! every unfiltered definition keeps its exact prior digest.
use super::entry::text;
use crate::application_query::ApplicationQueryWhereFilterProgram;
use worth_foundational::facade::CanonicalBasisEntry;

pub(super) fn append_where_filter(
    entries: &mut Vec<CanonicalBasisEntry>,
    filter: Option<&ApplicationQueryWhereFilterProgram>,
) {
    append_where_filter_at(entries, "where-filter", filter);
}

pub(super) fn append_where_filter_at(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    filter: Option<&ApplicationQueryWhereFilterProgram>,
) {
    let Some(filter) = filter else {
        return;
    };
    entries.push(text(
        format!("{prefix}.grammar"),
        "foundational-bool-expression-v1",
    ));
    for (index, operand) in filter.operands().iter().enumerate() {
        for (key, value) in operand.canonical_parts() {
            entries.push(text(
                format!("{prefix}.operand[{index}].{key}"),
                value.as_ref(),
            ));
        }
    }
    entries.push(text(
        format!("{prefix}.program"),
        filter.semantic_material(),
    ));
}
