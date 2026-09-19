#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiAppearanceStateAxisDemand(u8);

impl UiAppearanceStateAxisDemand {
    pub(crate) fn include(&mut self, axis: worth_ui_dsl::UiAppearanceStateAxis) {
        self.0 |= 1 << axis_index(axis);
    }

    pub(crate) const fn contains(self, axis: worth_ui_dsl::UiAppearanceStateAxis) -> bool {
        self.0 & (1 << axis_index(axis)) != 0
    }

    pub(crate) const fn index(axis: worth_ui_dsl::UiAppearanceStateAxis) -> usize {
        axis_index(axis) as usize
    }
}

const fn axis_index(axis: worth_ui_dsl::UiAppearanceStateAxis) -> u8 {
    match axis {
        worth_ui_dsl::UiAppearanceStateAxis::Operability => 0,
        worth_ui_dsl::UiAppearanceStateAxis::Focus => 1,
        worth_ui_dsl::UiAppearanceStateAxis::Validation => 2,
        worth_ui_dsl::UiAppearanceStateAxis::Selection => 3,
        worth_ui_dsl::UiAppearanceStateAxis::Hover => 4,
        worth_ui_dsl::UiAppearanceStateAxis::Pressed => 5,
    }
}
