#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceSupportPosture {
    Supported,
    Unsupported,
    Inapplicable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceProvenance {
    selected_slot: worth_ui_dsl::UiThemeSlotIdentity,
    terminal_slot: worth_ui_dsl::UiThemeSlotIdentity,
    source: Box<str>,
    aliases_compared: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiResolvedAppearanceAspect {
    aspect: worth_ui_dsl::UiAppearanceAspect,
    state_classes: Box<[worth_ui_dsl::UiAppearanceAxisClass]>,
    decision_cell_ordinal: u32,
    value: worth_ui_dsl::UiThemeValue,
    provenance: UiAppearanceProvenance,
    support: UiAppearanceSupportPosture,
    semantic_digest: u64,
    decision_cells_visited: u32,
    theme_slots_compared: u32,
}

impl UiAppearanceProvenance {
    pub(crate) fn new(
        selected_slot: worth_ui_dsl::UiThemeSlotIdentity,
        terminal_slot: worth_ui_dsl::UiThemeSlotIdentity,
        source: impl Into<Box<str>>,
        aliases_compared: u8,
    ) -> Self {
        Self {
            selected_slot,
            terminal_slot,
            source: source.into(),
            aliases_compared,
        }
    }

    pub(crate) fn selected_slot(&self) -> &worth_ui_dsl::UiThemeSlotIdentity {
        &self.selected_slot
    }
    pub(crate) fn terminal_slot(&self) -> &worth_ui_dsl::UiThemeSlotIdentity {
        &self.terminal_slot
    }
    pub(crate) fn source(&self) -> &str {
        &self.source
    }
    pub(crate) const fn aliases_compared(&self) -> u8 {
        self.aliases_compared
    }
}

impl UiResolvedAppearanceAspect {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        aspect: worth_ui_dsl::UiAppearanceAspect,
        state_classes: Box<[worth_ui_dsl::UiAppearanceAxisClass]>,
        decision_cell_ordinal: u32,
        value: worth_ui_dsl::UiThemeValue,
        provenance: UiAppearanceProvenance,
        support: UiAppearanceSupportPosture,
        semantic_digest: u64,
        decision_cells_visited: u32,
        theme_slots_compared: u32,
    ) -> Self {
        Self {
            aspect,
            state_classes,
            decision_cell_ordinal,
            value,
            provenance,
            support,
            semantic_digest,
            decision_cells_visited,
            theme_slots_compared,
        }
    }

    pub(crate) const fn aspect(&self) -> worth_ui_dsl::UiAppearanceAspect {
        self.aspect
    }
    pub(crate) fn state_classes(&self) -> &[worth_ui_dsl::UiAppearanceAxisClass] {
        &self.state_classes
    }
    pub(crate) const fn decision_cell_ordinal(&self) -> u32 {
        self.decision_cell_ordinal
    }
    pub(crate) const fn value(&self) -> worth_ui_dsl::UiThemeValue {
        self.value
    }
    pub(crate) const fn provenance(&self) -> &UiAppearanceProvenance {
        &self.provenance
    }
    pub(crate) const fn support(&self) -> UiAppearanceSupportPosture {
        self.support
    }
    pub(crate) const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
    pub(crate) const fn decision_cells_visited(&self) -> u32 {
        self.decision_cells_visited
    }
    pub(crate) const fn theme_slots_compared(&self) -> u32 {
        self.theme_slots_compared
    }

    pub(crate) fn exactly_equivalent(&self, other: &Self) -> bool {
        self == other
    }
}
