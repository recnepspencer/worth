#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenShapeCheck {
    lane: &'static str,
    actual_alignment: usize,
    expected_alignment: usize,
    actual_needs_drop: bool,
    expected_needs_drop: bool,
}

impl CodegenShapeCheck {
    pub fn new(
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

    pub fn lane(&self) -> &'static str {
        self.lane
    }

    pub fn matches(&self) -> bool {
        self.actual_alignment == self.expected_alignment
            && self.actual_needs_drop == self.expected_needs_drop
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenHonestyReport {
    suite: &'static str,
    verified_scope: &'static str,
    checks: Vec<CodegenShapeCheck>,
    hidden_dynamic_lookup: bool,
    hidden_virtual_dispatch: bool,
    mandatory_allocation_introduced: bool,
    residual_debt: &'static str,
}

impl CodegenHonestyReport {
    pub fn size_layout_and_drop_certified(
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

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn verified_scope(&self) -> &'static str {
        self.verified_scope
    }

    pub fn checks(&self) -> &[CodegenShapeCheck] {
        &self.checks
    }

    pub fn hidden_dynamic_lookup(&self) -> bool {
        self.hidden_dynamic_lookup
    }

    pub fn hidden_virtual_dispatch(&self) -> bool {
        self.hidden_virtual_dispatch
    }

    pub fn mandatory_allocation_introduced(&self) -> bool {
        self.mandatory_allocation_introduced
    }

    pub fn residual_debt(&self) -> &'static str {
        self.residual_debt
    }
}
