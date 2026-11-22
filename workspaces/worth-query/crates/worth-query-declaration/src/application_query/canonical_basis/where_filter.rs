//! Where-filter identity material. Absent filters contribute no entries, so
//! every unfiltered definition keeps its exact prior digest.
use super::entry::text;
use crate::application_query::ApplicationQueryWhereFilterProgram;
use worth_foundational::facade::CanonicalBasisEntry;

pub(super) fn append_where_filter(
    entries: &mut Vec<CanonicalBasisEntry>,
    filter: Option<&ApplicationQueryWhereFilterProgram>,
) {
    let Some(filter) = filter else {
        return;
    };
    entries.push(text(
        "where-filter.grammar",
        "foundational-bool-expression-v1",
    ));
    for (index, operand) in filter.operands().iter().enumerate() {
        for (key, value) in operand.canonical_parts() {
            entries.push(text(format!("where-filter.operand[{index}].{key}"), value));
        }
    }
    entries.push(text("where-filter.program", filter.semantic_material()));
}
