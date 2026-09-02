pub const UI_APPEARANCE_DECISION_CELL_CAPACITY: usize = 512;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiAppearanceStateAxis {
    Operability,
    Focus,
    Validation,
    Selection,
    Hover,
    Pressed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiAppearanceStateAxisVersion {
    axis: UiAppearanceStateAxis,
    revision: u16,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiAppearanceAxisClass {
    OperabilityReady,
    OperabilityPending,
    OperabilityOccupied,
    OperabilityDenied,
    OperabilityUnsupported,
    OperabilityStale,
    FocusUnfocused,
    FocusFocused,
    FocusVisible,
    FocusedWindowInactive,
    ValidationUnspecified,
    ValidationValid,
    ValidationAdvisory,
    ValidationInvalid,
    ValidationPending,
    ValidationStale,
    SelectionUnselected,
    SelectionSelected,
    SelectionAnchor,
    SelectionCursor,
    SelectedAnchorCursor,
    HoverOutside,
    Hovered,
    PressedIdle,
    PressedArmedInside,
    PressedCapturedOutside,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceAxisDomain {
    version: UiAppearanceStateAxisVersion,
    classes: Box<[UiAppearanceAxisClass]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UiAppearanceDecisionResult {
    value: UiAppearanceDecisionValue,
    value_kind: super::UiThemeValueKind,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum UiAppearanceDecisionValue {
    ThemeSlot(super::UiThemeSlotIdentity),
    Literal(super::UiThemeValue),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceDecisionRule {
    predicates: Box<[UiAppearanceAxisPredicate]>,
    result: UiAppearanceDecisionResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAppearanceAxisPredicate {
    axis: UiAppearanceStateAxis,
    class: Option<UiAppearanceAxisClass>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceDecisionCell {
    classes: Box<[UiAppearanceAxisClass]>,
    result: UiAppearanceDecisionResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceDecisionPartition {
    axes: Box<[UiAppearanceStateAxisVersion]>,
    cells: Box<[UiAppearanceDecisionCell]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceDecisionPartitionDenial {
    DuplicateAxis,
    PredicateArity,
    DuplicatePredicateAxis,
    MissingPredicateAxis,
    PredicateClassMismatch,
    CellCapacityExceeded,
    AmbiguousCell,
    OverlappingCell,
    DuplicateOtherwise,
    MissingCell,
    DuplicateCellName,
    MissingNamedCell,
    CyclicCellReference,
    ResultValueKindMismatch,
}

impl UiAppearanceStateAxisVersion {
    pub const fn current(axis: UiAppearanceStateAxis) -> Self {
        Self { axis, revision: 1 }
    }

    pub const fn axis(self) -> UiAppearanceStateAxis {
        self.axis
    }
    pub const fn revision(self) -> u16 {
        self.revision
    }
}

impl UiAppearanceAxisClass {
    pub const fn axis(self) -> UiAppearanceStateAxis {
        match self {
            Self::OperabilityReady
            | Self::OperabilityPending
            | Self::OperabilityOccupied
            | Self::OperabilityDenied
            | Self::OperabilityUnsupported
            | Self::OperabilityStale => UiAppearanceStateAxis::Operability,
            Self::FocusUnfocused
            | Self::FocusFocused
            | Self::FocusVisible
            | Self::FocusedWindowInactive => UiAppearanceStateAxis::Focus,
            Self::ValidationUnspecified
            | Self::ValidationValid
            | Self::ValidationAdvisory
            | Self::ValidationInvalid
            | Self::ValidationPending
            | Self::ValidationStale => UiAppearanceStateAxis::Validation,
            Self::SelectionUnselected
            | Self::SelectionSelected
            | Self::SelectionAnchor
            | Self::SelectionCursor
            | Self::SelectedAnchorCursor => UiAppearanceStateAxis::Selection,
            Self::HoverOutside | Self::Hovered => UiAppearanceStateAxis::Hover,
            Self::PressedIdle | Self::PressedArmedInside | Self::PressedCapturedOutside => {
                UiAppearanceStateAxis::Pressed
            }
        }
    }
}

impl UiAppearanceAxisDomain {
    pub fn complete(axis: UiAppearanceStateAxis) -> Self {
        Self {
            version: UiAppearanceStateAxisVersion::current(axis),
            classes: complete_classes(axis).into(),
        }
    }

    pub const fn version(&self) -> UiAppearanceStateAxisVersion {
        self.version
    }
    pub fn classes(&self) -> &[UiAppearanceAxisClass] {
        &self.classes
    }
}

fn complete_classes(axis: UiAppearanceStateAxis) -> &'static [UiAppearanceAxisClass] {
    use UiAppearanceAxisClass::*;
    match axis {
        UiAppearanceStateAxis::Operability => &[
            OperabilityReady,
            OperabilityPending,
            OperabilityOccupied,
            OperabilityDenied,
            OperabilityUnsupported,
            OperabilityStale,
        ],
        UiAppearanceStateAxis::Focus => &[
            FocusUnfocused,
            FocusFocused,
            FocusVisible,
            FocusedWindowInactive,
        ],
        UiAppearanceStateAxis::Validation => &[
            ValidationUnspecified,
            ValidationValid,
            ValidationAdvisory,
            ValidationInvalid,
            ValidationPending,
            ValidationStale,
        ],
        UiAppearanceStateAxis::Selection => &[
            SelectionUnselected,
            SelectionSelected,
            SelectionAnchor,
            SelectionCursor,
            SelectedAnchorCursor,
        ],
        UiAppearanceStateAxis::Hover => &[HoverOutside, Hovered],
        UiAppearanceStateAxis::Pressed => {
            &[PressedIdle, PressedArmedInside, PressedCapturedOutside]
        }
    }
}

impl UiAppearanceDecisionResult {
    pub fn theme_slot(
        slot: super::UiThemeSlotIdentity,
        value_kind: super::UiThemeValueKind,
    ) -> Self {
        Self {
            value: UiAppearanceDecisionValue::ThemeSlot(slot),
            value_kind,
        }
    }

    pub fn literal(value: super::UiThemeValue) -> Self {
        Self {
            value_kind: value.kind(),
            value: UiAppearanceDecisionValue::Literal(value),
        }
    }

    pub fn value(&self) -> &UiAppearanceDecisionValue {
        &self.value
    }

    pub fn slot(&self) -> Option<&super::UiThemeSlotIdentity> {
        match &self.value {
            UiAppearanceDecisionValue::ThemeSlot(slot) => Some(slot),
            UiAppearanceDecisionValue::Literal(_) => None,
        }
    }
    pub const fn value_kind(&self) -> super::UiThemeValueKind {
        self.value_kind
    }
}

impl UiAppearanceDecisionRule {
    pub fn new(
        predicates: impl IntoIterator<Item = UiAppearanceAxisPredicate>,
        result: UiAppearanceDecisionResult,
    ) -> Self {
        Self {
            predicates: predicates.into_iter().collect(),
            result,
        }
    }

    pub fn predicates(&self) -> &[UiAppearanceAxisPredicate] {
        &self.predicates
    }

    pub const fn result(&self) -> &UiAppearanceDecisionResult {
        &self.result
    }
}

impl UiAppearanceAxisPredicate {
    pub const fn any(axis: UiAppearanceStateAxis) -> Self {
        Self { axis, class: None }
    }

    pub const fn exact(class: UiAppearanceAxisClass) -> Self {
        Self {
            axis: class.axis(),
            class: Some(class),
        }
    }

    pub const fn axis(self) -> UiAppearanceStateAxis {
        self.axis
    }

    pub const fn class(self) -> Option<UiAppearanceAxisClass> {
        self.class
    }
}

#[path = "state_partition_compilation.rs"]
mod compilation;

#[cfg(test)]
fn admit_cell_count(
    cardinalities: impl IntoIterator<Item = usize>,
) -> Result<usize, UiAppearanceDecisionPartitionDenial> {
    compilation::admit_cell_count(cardinalities)
}

#[cfg(test)]
#[path = "state_partition_tests.rs"]
mod tests;
