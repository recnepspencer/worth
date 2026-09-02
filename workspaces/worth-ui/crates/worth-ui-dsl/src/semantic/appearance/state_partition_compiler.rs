use std::collections::BTreeMap;

#[path = "state_partition_compiler_support.rs"]
mod support;
use support::{complete_predicates, expand_cells, reject_overlaps, rule_matches};

use super::{
    UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial, UiAppearanceDecisionResult,
    UiAppearanceDecisionRule, UiThemeSlotIdentity, UiThemeValue, UiThemeValueKind,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceCell {
    name: Option<Box<str>>,
    predicates: Box<[UiAppearanceAxisPredicate]>,
    value: UiAppearanceCellValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAppearanceCellValue {
    ThemeSlot {
        slot: UiThemeSlotIdentity,
        value_kind: UiThemeValueKind,
    },
    Literal(UiThemeValue),
    SameAs(Box<str>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceCellBuilderDenial {
    InvalidThemeSlotIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearancePartitionAuthoring {
    domains: Vec<UiAppearanceAxisDomain>,
    cells: Vec<UiAppearanceCell>,
    otherwise: Option<UiAppearanceCellValue>,
    duplicate_otherwise: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceCellBuilder {
    name: Option<Box<str>>,
    predicates: Vec<UiAppearanceAxisPredicate>,
}

impl UiAppearanceCell {
    pub fn new(
        name: Option<impl Into<Box<str>>>,
        predicates: impl IntoIterator<Item = UiAppearanceAxisPredicate>,
        value: UiAppearanceCellValue,
    ) -> Self {
        Self {
            name: name.map(Into::into),
            predicates: predicates
                .into_iter()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            value,
        }
    }

    pub fn named(name: impl Into<Box<str>>) -> UiAppearanceCellBuilder {
        UiAppearanceCellBuilder {
            name: Some(name.into()),
            predicates: Vec::new(),
        }
    }

    pub fn when(
        predicates: impl IntoIterator<Item = UiAppearanceAxisPredicate>,
    ) -> UiAppearanceCellBuilder {
        UiAppearanceCellBuilder {
            name: None,
            predicates: predicates.into_iter().collect(),
        }
    }

    pub fn otherwise_same_as(name: impl Into<Box<str>>) -> Self {
        Self::new(
            None::<Box<str>>,
            [],
            UiAppearanceCellValue::SameAs(name.into()),
        )
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn predicates(&self) -> &[UiAppearanceAxisPredicate] {
        &self.predicates
    }

    pub fn value(&self) -> &UiAppearanceCellValue {
        &self.value
    }
}

impl UiAppearanceCellBuilder {
    pub fn when(mut self, predicates: impl IntoIterator<Item = UiAppearanceAxisPredicate>) -> Self {
        self.predicates = predicates.into_iter().collect();
        self
    }

    pub fn uses_slot(
        self,
        slot: UiThemeSlotIdentity,
        value_kind: UiThemeValueKind,
    ) -> UiAppearanceCell {
        UiAppearanceCell::new(
            self.name,
            self.predicates,
            UiAppearanceCellValue::ThemeSlot { slot, value_kind },
        )
    }

    pub fn uses_color(
        self,
        slot: impl Into<String>,
    ) -> Result<UiAppearanceCell, UiAppearanceCellBuilderDenial> {
        UiThemeSlotIdentity::new(slot.into())
            .map(|slot| self.uses_slot(slot, UiThemeValueKind::Color))
            .ok_or(UiAppearanceCellBuilderDenial::InvalidThemeSlotIdentity)
    }

    pub fn uses(self, value: UiAppearanceCellValue) -> UiAppearanceCell {
        UiAppearanceCell::new(self.name, self.predicates, value)
    }

    pub fn literal(self, value: UiThemeValue) -> UiAppearanceCell {
        self.uses(UiAppearanceCellValue::Literal(value))
    }

    pub fn same_as(self, name: impl Into<Box<str>>) -> UiAppearanceCell {
        self.uses(UiAppearanceCellValue::SameAs(name.into()))
    }
}

impl UiAppearanceCellValue {
    pub fn theme_slot(slot: UiThemeSlotIdentity, value_kind: UiThemeValueKind) -> Self {
        Self::ThemeSlot { slot, value_kind }
    }

    pub fn literal(value: UiThemeValue) -> Self {
        Self::Literal(value)
    }

    pub fn same_as(name: impl Into<Box<str>>) -> Self {
        Self::SameAs(name.into())
    }
}

impl UiAppearancePartitionAuthoring {
    pub fn new(domains: impl IntoIterator<Item = UiAppearanceAxisDomain>) -> Self {
        Self {
            domains: domains.into_iter().collect(),
            cells: Vec::new(),
            otherwise: None,
            duplicate_otherwise: false,
        }
    }

    pub fn with_cell(mut self, cell: UiAppearanceCell) -> Self {
        self.cells.push(cell);
        self
    }

    pub fn with_cells(mut self, cells: impl IntoIterator<Item = UiAppearanceCell>) -> Self {
        self.cells.extend(cells);
        self
    }

    pub fn with_otherwise(mut self, value: UiAppearanceCellValue) -> Self {
        if self.otherwise.replace(value).is_some() {
            self.duplicate_otherwise = true;
        }
        self
    }

    pub fn otherwise_same_as(self, name: impl Into<Box<str>>) -> Self {
        self.with_otherwise(UiAppearanceCellValue::SameAs(name.into()))
    }

    pub fn domains(&self) -> &[UiAppearanceAxisDomain] {
        &self.domains
    }

    pub fn cells(&self) -> &[UiAppearanceCell] {
        &self.cells
    }

    pub fn compile(
        &self,
        aspect: super::UiAppearanceAspect,
    ) -> Result<UiAppearanceDecisionPartition, UiAppearanceDecisionPartitionDenial> {
        let mut domains = self.domains.clone();
        domains.sort_by_key(UiAppearanceAxisDomain::version);
        if domains
            .windows(2)
            .any(|pair| pair[0].version().axis() == pair[1].version().axis())
        {
            return Err(UiAppearanceDecisionPartitionDenial::DuplicateAxis);
        }
        let capacity = domains
            .iter()
            .map(|domain| domain.classes().len())
            .try_fold(1_usize, usize::checked_mul)
            .ok_or(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded)?;
        if capacity > super::UI_APPEARANCE_DECISION_CELL_CAPACITY {
            return Err(UiAppearanceDecisionPartitionDenial::CellCapacityExceeded);
        }
        if self.duplicate_otherwise {
            return Err(UiAppearanceDecisionPartitionDenial::DuplicateOtherwise);
        }

        let names = named_cells(&self.cells)?;
        let mut resolving = vec![ResolveState::Unvisited; self.cells.len()];
        let results = (0..self.cells.len())
            .map(|index| resolve_cell(index, &self.cells, &names, &mut resolving))
            .collect::<Result<Vec<_>, _>>()?;
        for result in &results {
            if result.value_kind() != aspect.value_kind() {
                return Err(UiAppearanceDecisionPartitionDenial::ResultValueKindMismatch);
            }
        }

        let rules = self
            .cells
            .iter()
            .zip(results.iter())
            .map(|(cell, result)| {
                Ok(UiAppearanceDecisionRule::new(
                    complete_predicates(&domains, cell.predicates())?,
                    result.clone(),
                ))
            })
            .collect::<Result<Vec<_>, UiAppearanceDecisionPartitionDenial>>()?;
        reject_overlaps(&domains, &rules)?;

        let mut completed_rules = rules;
        if let Some(otherwise) = &self.otherwise {
            let otherwise_result =
                resolve_otherwise(otherwise, &self.cells, &names, &mut resolving)?;
            if otherwise_result.value_kind() != aspect.value_kind() {
                return Err(UiAppearanceDecisionPartitionDenial::ResultValueKindMismatch);
            }
            expand_cells(&domains, &mut Vec::new(), &mut |classes| {
                if !completed_rules
                    .iter()
                    .any(|rule| rule_matches(rule, classes))
                {
                    completed_rules.push(UiAppearanceDecisionRule::new(
                        classes
                            .iter()
                            .copied()
                            .map(UiAppearanceAxisPredicate::exact),
                        otherwise_result.clone(),
                    ));
                }
                Ok(())
            })?;
        }
        UiAppearanceDecisionPartition::compile(domains, completed_rules)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ResolveState {
    Unvisited,
    Visiting,
    Resolved,
}

fn named_cells(
    cells: &[UiAppearanceCell],
) -> Result<BTreeMap<&str, usize>, UiAppearanceDecisionPartitionDenial> {
    let mut names = BTreeMap::new();
    for (index, cell) in cells.iter().enumerate() {
        if let Some(name) = cell.name() {
            if names.insert(name, index).is_some() {
                return Err(UiAppearanceDecisionPartitionDenial::DuplicateCellName);
            }
        }
    }
    Ok(names)
}

fn resolve_cell(
    index: usize,
    cells: &[UiAppearanceCell],
    names: &BTreeMap<&str, usize>,
    states: &mut [ResolveState],
) -> Result<UiAppearanceDecisionResult, UiAppearanceDecisionPartitionDenial> {
    match states[index] {
        ResolveState::Resolved => {}
        ResolveState::Visiting => {
            return Err(UiAppearanceDecisionPartitionDenial::CyclicCellReference)
        }
        ResolveState::Unvisited => states[index] = ResolveState::Visiting,
    }
    let result = match &cells[index].value {
        UiAppearanceCellValue::ThemeSlot { slot, value_kind } => {
            UiAppearanceDecisionResult::theme_slot(slot.clone(), *value_kind)
        }
        UiAppearanceCellValue::Literal(value) => UiAppearanceDecisionResult::literal(*value),
        UiAppearanceCellValue::SameAs(name) => {
            let target = names
                .get(name.as_ref())
                .copied()
                .ok_or(UiAppearanceDecisionPartitionDenial::MissingNamedCell)?;
            resolve_cell(target, cells, names, states)?
        }
    };
    states[index] = ResolveState::Resolved;
    Ok(result)
}

fn resolve_otherwise(
    value: &UiAppearanceCellValue,
    cells: &[UiAppearanceCell],
    names: &BTreeMap<&str, usize>,
    states: &mut [ResolveState],
) -> Result<UiAppearanceDecisionResult, UiAppearanceDecisionPartitionDenial> {
    match value {
        UiAppearanceCellValue::SameAs(name) => {
            let index = names
                .get(name.as_ref())
                .copied()
                .ok_or(UiAppearanceDecisionPartitionDenial::MissingNamedCell)?;
            resolve_cell(index, cells, names, states)
        }
        _ => value_to_result(value),
    }
}

fn value_to_result(
    value: &UiAppearanceCellValue,
) -> Result<UiAppearanceDecisionResult, UiAppearanceDecisionPartitionDenial> {
    Ok(match value {
        UiAppearanceCellValue::ThemeSlot { slot, value_kind } => {
            UiAppearanceDecisionResult::theme_slot(slot.clone(), *value_kind)
        }
        UiAppearanceCellValue::Literal(value) => UiAppearanceDecisionResult::literal(*value),
        UiAppearanceCellValue::SameAs(_) => {
            return Err(UiAppearanceDecisionPartitionDenial::MissingNamedCell)
        }
    })
}
