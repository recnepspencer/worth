use super::{
    UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceDecisionCell,
    UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial, UiAppearanceDecisionResult,
    UiAppearanceDecisionRule, UiAppearanceStateAxisVersion, UI_APPEARANCE_DECISION_CELL_CAPACITY,
};

impl UiAppearanceDecisionPartition {
    pub fn compile(
        domains: impl IntoIterator<Item = UiAppearanceAxisDomain>,
        rules: impl IntoIterator<Item = UiAppearanceDecisionRule>,
    ) -> Result<Self, UiAppearanceDecisionPartitionDenial> {
        let mut domains = domains.into_iter().collect::<Vec<_>>();
        domains.sort_by_key(UiAppearanceAxisDomain::version);
        if domains
            .windows(2)
            .any(|pair| pair[0].version.axis == pair[1].version.axis)
        {
            return Err(UiAppearanceDecisionPartitionDenial::DuplicateAxis);
        }
        let cell_count = admit_cell_count(domains.iter().map(|domain| domain.classes.len()))?;
        let rules = rules.into_iter().collect::<Vec<_>>();
        validate_rules(&domains, &rules)?;
        let mut cells = Vec::with_capacity(cell_count);
        expand_cells(&domains, 0, &mut Vec::new(), &mut |classes| {
            let matches = rules
                .iter()
                .filter(|rule| rule_matches(rule, classes))
                .collect::<Vec<_>>();
            match matches.as_slice() {
                [] => Err(UiAppearanceDecisionPartitionDenial::MissingCell),
                [rule] => {
                    cells.push(UiAppearanceDecisionCell {
                        classes: classes.to_vec().into_boxed_slice(),
                        result: rule.result.clone(),
                    });
                    Ok(())
                }
                _ => Err(UiAppearanceDecisionPartitionDenial::AmbiguousCell),
            }
        })?;
        Ok(Self {
            axes: domains
                .iter()
                .map(UiAppearanceAxisDomain::version)
                .collect(),
            cells: cells.into_boxed_slice(),
        })
    }

    pub fn axes(&self) -> &[UiAppearanceStateAxisVersion] {
        &self.axes
    }

    pub fn cells(&self) -> &[UiAppearanceDecisionCell] {
        &self.cells
    }
}

pub(super) fn admit_cell_count(
    cardinalities: impl IntoIterator<Item = usize>,
) -> Result<usize, UiAppearanceDecisionPartitionDenial> {
    let count = cardinalities
        .into_iter()
        .try_fold(1_usize, usize::checked_mul)
        .ok_or(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)?;
    if count > UI_APPEARANCE_DECISION_CELL_CAPACITY {
        Err(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)
    } else {
        Ok(count)
    }
}

fn validate_rules(
    domains: &[UiAppearanceAxisDomain],
    rules: &[UiAppearanceDecisionRule],
) -> Result<(), UiAppearanceDecisionPartitionDenial> {
    for rule in rules {
        if rule.predicates.len() != domains.len() {
            return Err(UiAppearanceDecisionPartitionDenial::PredicateArity);
        }
        let mut axes = rule
            .predicates
            .iter()
            .map(|predicate| predicate.axis)
            .collect::<Vec<_>>();
        axes.sort_unstable();
        if axes.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(UiAppearanceDecisionPartitionDenial::DuplicatePredicateAxis);
        }
        for domain in domains {
            let predicate = rule
                .predicates
                .iter()
                .find(|predicate| predicate.axis == domain.version.axis)
                .ok_or(UiAppearanceDecisionPartitionDenial::MissingPredicateAxis)?;
            if predicate
                .class
                .is_some_and(|class| !domain.classes.contains(&class))
            {
                return Err(UiAppearanceDecisionPartitionDenial::PredicateClassMismatch);
            }
        }
    }
    Ok(())
}

fn expand_cells(
    domains: &[UiAppearanceAxisDomain],
    index: usize,
    current: &mut Vec<UiAppearanceAxisClass>,
    emit: &mut impl FnMut(&[UiAppearanceAxisClass]) -> Result<(), UiAppearanceDecisionPartitionDenial>,
) -> Result<(), UiAppearanceDecisionPartitionDenial> {
    if index == domains.len() {
        return emit(current);
    }
    for class in domains[index].classes.iter().copied() {
        current.push(class);
        expand_cells(domains, index + 1, current, emit)?;
        current.pop();
    }
    Ok(())
}

fn rule_matches(rule: &UiAppearanceDecisionRule, cell: &[UiAppearanceAxisClass]) -> bool {
    cell.iter().all(|class| {
        rule.predicates
            .iter()
            .find(|predicate| predicate.axis == class.axis())
            .is_some_and(|predicate| predicate.class.is_none_or(|value| value == *class))
    })
}

impl UiAppearanceDecisionCell {
    pub fn classes(&self) -> &[UiAppearanceAxisClass] {
        &self.classes
    }

    pub const fn result(&self) -> &UiAppearanceDecisionResult {
        &self.result
    }
}
