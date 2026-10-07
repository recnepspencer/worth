/// A named implementation home that Milestone 1 expects to remain distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponsibilityArea {
    name: &'static str,
    owns: &'static str,
    does_not_own: &'static str,
}

impl ResponsibilityArea {
    pub const fn new(name: &'static str, owns: &'static str, does_not_own: &'static str) -> Self {
        Self {
            name,
            owns,
            does_not_own,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn owns(&self) -> &'static str {
        self.owns
    }

    pub const fn does_not_own(&self) -> &'static str {
        self.does_not_own
    }
}

/// Responsibility topology exposed for Phase 1 certification.
pub fn foundational_responsibilities() -> [ResponsibilityArea; 14] {
    [
        crate::values::responsibility(),
        crate::aspects::responsibility(),
        crate::identities::responsibility(),
        crate::locators::responsibility(),
        crate::compatibility::responsibility(),
        crate::canonicalization::responsibility(),
        crate::profiles::responsibility(),
        crate::boundary_artifacts::responsibility(),
        crate::transitions::responsibility(),
        crate::diagnostics::responsibility(),
        crate::boundary_evidence::responsibility(),
        crate::performance::responsibility(),
        crate::budget_limits::responsibility(),
        crate::expressions::responsibility(),
    ]
}
