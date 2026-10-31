//! One disclosure compiler; diagnostic callers admit the work it actually owns.
use super::admission::{pay, unmetered, CompilationStop, CompilationWork};
use super::field_masks::admit_field_masks;
use super::result_rules::{admit_result_rule, require_complete_result_shape};
use super::*;

pub(in crate::domain_computation) fn compile_disclosure_contract<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<
    WorthQueryAdmittedApplicationDisclosureContract,
    WorthQueryApplicationDisclosureContractDenial,
> {
    unmetered(compile_with_admission(query, layout, &mut |_| {
        Ok::<_, std::convert::Infallible>(())
    }))
}

pub(super) fn compile_with_admission<Schema, Query, Parameters, QueryResult, Scope, E>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<WorthQueryAdmittedApplicationDisclosureContract, CompilationStop<E>> {
    pay(admit, CompilationWork::Visit)?;
    match query.disclosure().posture() {
        ApplicationQueryDisclosurePosture::Public if query.disclosure().rules().is_empty() => {
            return Ok(WorthQueryAdmittedApplicationDisclosureContract::Public);
        }
        ApplicationQueryDisclosurePosture::Governed => {}
        _ => return Err(denial(query.name()).into()),
    }
    let mut result_rules = BTreeMap::new();
    let (count, group_counts) = internal_rule_counts(query.disclosure().rules(), admit)?;
    let mut internal_rules = Vec::with_capacity(count);
    let mut internal_field_rules = BTreeMap::new();
    for rule in query.disclosure().rules() {
        pay(admit, CompilationWork::Visit)?;
        admit_rule(
            query,
            layout,
            rule,
            &mut result_rules,
            &mut internal_rules,
            &mut internal_field_rules,
            &group_counts,
            admit,
        )?;
    }
    require_complete_result_shape(query, &result_rules)?;
    validate_influence(query, &internal_field_rules, &result_rules, admit)?;
    let contract = query.disclosure();
    let capability_name = contract
        .capability_name()
        .ok_or_else(|| denial(query.name()))?;
    let capability_type = contract
        .capability_type()
        .ok_or_else(|| denial(query.name()))?;
    for text in [contract.classification(), capability_name, capability_type] {
        pay(admit, CompilationWork::Copy(text.len()))?;
    }
    Ok(WorthQueryAdmittedApplicationDisclosureContract::Governed {
        classification: contract.classification().to_string(),
        capability_name: capability_name.to_string(),
        capability_type: capability_type.to_string(),
        result_rules,
        internal_rules,
        internal_field_rules,
    })
}

#[allow(clippy::too_many_arguments)]
fn admit_rule<Schema, Query, Parameters, QueryResult, Scope, E>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    layout: &WorthQueryPrimaryGraphLayout,
    rule: &ApplicationQueryDisclosureRule,
    result_rules: &mut BTreeMap<ApplicationQueryResultSlotKey, AdmittedDisclosureRule>,
    internal_rules: &mut Vec<AdmittedDisclosureRule>,
    internal_field_rules: &mut GovernedInternalFieldRules,
    group_counts: &InternalRuleCounts<'_>,
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(), CompilationStop<E>> {
    let field = rule
        .selector()
        .field_contract()
        .map(|field| admit_field_masks(layout, rule.selector(), field, admit))
        .transpose()?;
    pay(
        admit,
        CompilationWork::Copy(rule.disclosure_value().owned_allocation_capacity_bytes()),
    )?;
    pay(
        admit,
        CompilationWork::Scan {
            entries: rule.influence().permitted().len(),
            parts: &[],
        },
    )?;
    let installed = Arc::new(WorthQueryAdmittedApplicationDisclosureRule {
        disclosure_value: rule.disclosure_value().clone(),
        influence: rule.influence().clone(),
        field,
    });
    match rule.selector() {
        ApplicationQueryDisclosureSelector::InspectionMetadata(metadata) => {
            return Err(denial(metadata.slot_type()).into())
        }
        ApplicationQueryDisclosureSelector::Resource(meaning) => {
            return Err(denial(meaning.slot_type()).into())
        }
        ApplicationQueryDisclosureSelector::HistoryMetadata(metadata) => {
            return Err(denial(metadata.slot_type()).into())
        }
        ApplicationQueryDisclosureSelector::InternalField { .. } => {
            let field = installed
                .field()
                .ok_or_else(|| denial(rule.selector().slot_type()))?;
            let parts = [&*field.entity, &*field.aspect, &*field.field];
            pay(
                admit,
                CompilationWork::Lookup {
                    entries: internal_field_rules.len(),
                    parts: &parts,
                },
            )?;
            for text in parts {
                pay(admit, CompilationWork::Copy(text.len()))?;
            }
            let key = (
                field.entity.clone(),
                field.aspect.clone(),
                field.field.clone(),
            );
            let group = match internal_field_rules.entry(key) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::btree_map::Entry::Vacant(entry) => {
                    pay(
                        admit,
                        CompilationWork::Lookup {
                            entries: group_counts.len(),
                            parts: &parts,
                        },
                    )?;
                    let count = group_counts
                        .get(&(parts[0], parts[1], parts[2]))
                        .copied()
                        .ok_or_else(|| denial(rule.selector().slot_type()))?;
                    entry.insert(Vec::with_capacity(count))
                }
            };
            group.push(Arc::clone(&installed));
            internal_rules.push(installed);
        }
        ApplicationQueryDisclosureSelector::Field { .. } => {
            admit_result_rule(query, rule.selector(), installed, result_rules, true, admit)?
        }
        ApplicationQueryDisclosureSelector::Relation { .. } => admit_result_rule(
            query,
            rule.selector(),
            installed,
            result_rules,
            false,
            admit,
        )?,
        ApplicationQueryDisclosureSelector::Components(selection) => {
            let slot = selection.slot_key();
            if !query
                .read_graph()
                .component_selection()
                .is_some_and(|expected| expected.slot_key() == slot)
                || result_rules.insert(slot, installed).is_some()
            {
                return Err(denial(selection.slot_type()).into());
            }
        }
        ApplicationQueryDisclosureSelector::Orbit(orbit) => {
            let slot = orbit.slot_key();
            if query
                .read_graph()
                .orbit()
                .is_none_or(|expected| expected.slot_key() != slot)
                || result_rules.insert(slot, installed).is_some()
            {
                return Err(denial(orbit.slot_type()).into());
            }
        }
    }
    Ok(())
}

// Borrowed immutable names are capacity bookkeeping, never copied authority.
type InternalRuleCounts<'a> = BTreeMap<(&'a str, &'a str, &'a str), usize>;

fn internal_rule_counts<'a, E>(
    rules: &'a [ApplicationQueryDisclosureRule],
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
) -> Result<(usize, InternalRuleCounts<'a>), CompilationStop<E>> {
    let mut count = 0;
    let mut groups = BTreeMap::new();
    for rule in rules {
        pay(admit, CompilationWork::Visit)?;
        if !matches!(
            rule.selector(),
            ApplicationQueryDisclosureSelector::InternalField { .. }
        ) {
            continue;
        }
        let field = rule
            .selector()
            .field_contract()
            .ok_or_else(|| denial(rule.selector().slot_type()))?;
        pay(
            admit,
            CompilationWork::Lookup {
                entries: groups.len(),
                parts: &[field.0, field.1, field.2],
            },
        )?;
        *groups.entry(field).or_insert(0) += 1;
        count += 1; // Both counters are subsets of this actual slice.
    }
    Ok((count, groups))
}
