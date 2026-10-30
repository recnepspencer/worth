#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodegenShapeCheck {
    pub(crate) lane: &'static str,
    pub(crate) actual_alignment: usize,
    pub(crate) expected_alignment: usize,
    pub(crate) actual_needs_drop: bool,
    pub(crate) expected_needs_drop: bool,
}

impl CodegenShapeCheck {
    pub(crate) fn new(
        lane: &'static str,
        actual_alignment: usize,
        expected_alignment: usize,
        actual_needs_drop: bool,
        expected_needs_drop: bool,
    ) -> Self {
        Self {
            lane,
            actual_alignment,
            expected_alignment,
            actual_needs_drop,
            expected_needs_drop,
        }
    }

    pub(crate) fn lane(&self) -> &'static str {
        self.lane
    }

    pub(crate) fn matches(&self) -> bool {
        self.actual_alignment == self.expected_alignment
            && self.actual_needs_drop == self.expected_needs_drop
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodegenHonestyReport {
    pub(crate) suite: &'static str,
    pub(crate) verified_scope: &'static str,
    pub(crate) checks: Vec<CodegenShapeCheck>,
    pub(crate) hidden_dynamic_lookup: bool,
    pub(crate) hidden_virtual_dispatch: bool,
    pub(crate) mandatory_allocation_introduced: bool,
    pub(crate) residual_debt: &'static str,
}

impl CodegenHonestyReport {
    pub(crate) fn size_layout_and_drop_certified(
        suite: &'static str,
        checks: Vec<CodegenShapeCheck>,
        residual_debt: &'static str,
    ) -> Self {
        Self {
            suite,
            verified_scope: "size_layout_and_drop_only",
            checks,
            hidden_dynamic_lookup: false,
            hidden_virtual_dispatch: false,
            mandatory_allocation_introduced: false,
            residual_debt,
        }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn verified_scope(&self) -> &'static str {
        self.verified_scope
    }

    pub(crate) fn checks(&self) -> &[CodegenShapeCheck] {
        &self.checks
    }

    pub(crate) fn hidden_dynamic_lookup(&self) -> bool {
        self.hidden_dynamic_lookup
    }

    pub(crate) fn hidden_virtual_dispatch(&self) -> bool {
        self.hidden_virtual_dispatch
    }

    pub(crate) fn mandatory_allocation_introduced(&self) -> bool {
        self.mandatory_allocation_introduced
    }

    pub(crate) fn residual_debt(&self) -> &'static str {
        self.residual_debt
    }
}
