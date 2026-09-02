use super::{
    UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceDecisionPartitionDenial, UiAppearanceDecisionRule,
};

pub(super) fn complete_predicates(
    domains: &[UiAppearanceAxisDomain],
    predicates: &[UiAppearanceAxisPredicate],
) -> Result<Vec<UiAppearanceAxisPredicate>, UiAppearanceDecisionPartitionDenial> {
    let mut seen = Vec::new();
    for predicate in predicates {
        if !domains
            .iter()
            .any(|domain| domain.version().axis() == predicate.axis())
        {
            return Err(UiAppearanceDecisionPartitionDenial::MissingPredicateAxis);
        }
        if seen.contains(&predicate.axis()) {
            return Err(UiAppearanceDecisionPartitionDenial::DuplicatePredicateAxis);
        }
        if let Some(class) = predicate.class() {
            let domain = domains
                .iter()
                .find(|domain| domain.version().axis() == predicate.axis())
                .expect("predicate axis was validated");
            if !domain.classes().contains(&class) {
                return Err(UiAppearanceDecisionPartitionDenial::PredicateClassMismatch);
            }
        }
        seen.push(predicate.axis());
    }
    Ok(domains
        .iter()
        .map(|domain| {
            predicates
                .iter()
                .find(|predicate| predicate.axis() == domain.version().axis())
                .copied()
                .unwrap_or_else(|| UiAppearanceAxisPredicate::any(domain.version().axis()))
        })
        .collect())
}

pub(super) fn reject_overlaps(
    domains: &[UiAppearanceAxisDomain],
    rules: &[UiAppearanceDecisionRule],
) -> Result<(), UiAppearanceDecisionPartitionDenial> {
    expand_cells(domains, &mut Vec::new(), &mut |classes| {
        if rules
            .iter()
            .filter(|rule| rule_matches(rule, classes))
            .count()
            > 1
        {
            Err(UiAppearanceDecisionPartitionDenial::OverlappingCell)
        } else {
            Ok(())
        }
    })
}

pub(super) fn expand_cells(
    domains: &[UiAppearanceAxisDomain],
    current: &mut Vec<UiAppearanceAxisClass>,
    emit: &mut impl FnMut(&[UiAppearanceAxisClass]) -> Result<(), UiAppearanceDecisionPartitionDenial>,
) -> Result<(), UiAppearanceDecisionPartitionDenial> {
    if current.len() == domains.len() {
        return emit(current);
    }
    for class in domains[current.len()].classes() {
        current.push(*class);
        expand_cells(domains, current, emit)?;
        current.pop();
    }
    Ok(())
}

pub(super) fn rule_matches(
    rule: &UiAppearanceDecisionRule,
    classes: &[UiAppearanceAxisClass],
) -> bool {
    classes.iter().all(|class| {
        rule.predicates()
            .iter()
            .find(|predicate| predicate.axis() == class.axis())
            .is_some_and(|predicate| predicate.class().is_none_or(|expected| expected == *class))
    })
}
