//! Exact declared output-slot membership and completeness for the one compiler.
use super::admission::{pay, CompilationStop, CompilationWork};
use super::*;

pub(in crate::domain_computation::authorization::application_disclosure) fn selector_parts(
    selector: &ApplicationQueryDisclosureSelector,
) -> [&str; 7] {
    match selector {
        ApplicationQueryDisclosureSelector::Field {
            query_type,
            slot_type,
            entity,
            aspect,
            field,
            output_name,
            value_type,
            ..
        } => [
            query_type.as_str(),
            slot_type.as_str(),
            entity,
            aspect,
            field,
            output_name,
            value_type.as_str(),
        ],
        ApplicationQueryDisclosureSelector::Relation {
            query_type,
            slot_type,
            relation,
            from,
            to,
            output_name,
            ..
        } => [
            query_type.as_str(),
            slot_type.as_str(),
            relation,
            from,
            to,
            output_name,
            "",
        ],
        _ => [
            selector.query_type(),
            selector.slot_type(),
            selector.output_name(),
            "",
            "",
            "",
            "",
        ],
    }
}

pub(super) fn admit_result_rule<Schema, Query, Parameters, QueryResult, Scope, E>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    selector: &ApplicationQueryDisclosureSelector,
    rule: AdmittedDisclosureRule,
    result_rules: &mut BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    field: bool,
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(), CompilationStop<E>> {
    let parts = selector_parts(selector);
    for text in parts {
        pay(admit, CompilationWork::Copy(text.len()))?;
    }
    let slot = selector
        .result_slot_key()
        .ok_or_else(|| denial(selector.slot_type()))?;
    let entries = if field {
        query.read_graph().projections().len()
    } else {
        query.read_graph().relations().len()
    };
    pay(
        admit,
        CompilationWork::Scan {
            entries,
            parts: &parts,
        },
    )?;
    let exact = if field {
        query
            .read_graph()
            .projections()
            .iter()
            .any(|projection| projection.slot_key_identity().as_ref() == &slot)
    } else {
        query
            .read_graph()
            .relations()
            .iter()
            .any(|relation| relation.slot_key_identity().as_ref() == &slot)
    };
    if exact {
        pay(
            admit,
            CompilationWork::Lookup {
                entries: result_rules.len(),
                parts: &parts,
            },
        )?;
        if result_rules.insert(slot, rule).is_none() {
            return Ok(());
        }
    }
    Err(denial(selector.slot_type()).into())
}

pub(super) fn require_complete_result_shape<Schema, Query, Parameters, QueryResult, Scope>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    rules: &BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
) -> Result<(), WorthQueryApplicationDisclosureContractDenial> {
    let graph = query.read_graph();
    let expected = graph
        .projections()
        .len()
        .checked_add(graph.relations().len())
        .and_then(|n| n.checked_add(usize::from(graph.orbit().is_some())))
        .and_then(|n| n.checked_add(usize::from(graph.component_selection().is_some())));
    // Every insertion first proves actual graph membership and refuses a
    // duplicate key. Equal cardinality therefore proves complete coverage.
    (expected == Some(rules.len()))
        .then_some(())
        .ok_or_else(|| denial(query.name()))
}
