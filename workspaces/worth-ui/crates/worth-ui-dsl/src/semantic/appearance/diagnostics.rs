#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiAppearanceCellReferenceOrigin {
    NamedCell,
    OtherwiseClause,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UiAppearanceCanonicalStateCell {
    classes: Box<[super::UiAppearanceAxisClass]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UiAppearanceFinitePredicate {
    predicates: Box<[super::UiAppearanceAxisPredicate]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearancePartitionAdmissionDenial {
    role: super::UiAppearanceRoleIdentity,
    aspect: super::UiAppearanceAspect,
    expected_kind: super::UiThemeValueKind,
    denial: super::UiAppearanceDecisionPartitionDenial,
}

impl UiAppearanceCanonicalStateCell {
    pub fn new(classes: impl IntoIterator<Item = super::UiAppearanceAxisClass>) -> Self {
        Self {
            classes: classes.into_iter().collect(),
        }
    }

    pub fn classes(&self) -> &[super::UiAppearanceAxisClass] {
        &self.classes
    }
}

impl UiAppearanceFinitePredicate {
    pub fn exact_for(cell: &UiAppearanceCanonicalStateCell) -> Self {
        Self {
            predicates: cell
                .classes()
                .iter()
                .copied()
                .map(super::UiAppearanceAxisPredicate::exact)
                .collect(),
        }
    }

    pub fn predicates(&self) -> &[super::UiAppearanceAxisPredicate] {
        &self.predicates
    }
}

impl UiAppearancePartitionAdmissionDenial {
    pub(crate) fn new(
        role: super::UiAppearanceRoleIdentity,
        aspect: super::UiAppearanceAspect,
        expected_kind: super::UiThemeValueKind,
        denial: super::UiAppearanceDecisionPartitionDenial,
    ) -> Self {
        Self {
            role,
            aspect,
            expected_kind,
            denial,
        }
    }

    pub fn role(&self) -> &super::UiAppearanceRoleIdentity {
        &self.role
    }

    pub const fn aspect(&self) -> super::UiAppearanceAspect {
        self.aspect
    }

    pub const fn expected_kind(&self) -> super::UiThemeValueKind {
        self.expected_kind
    }

    pub fn denial(&self) -> &super::UiAppearanceDecisionPartitionDenial {
        &self.denial
    }
}
