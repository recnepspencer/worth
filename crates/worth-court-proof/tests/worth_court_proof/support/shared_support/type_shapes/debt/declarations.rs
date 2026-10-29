#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebtItem {
    pub(crate) category: &'static str,
    pub(crate) debt: &'static str,
}

impl DebtItem {
    pub(crate) fn new(category: &'static str, debt: &'static str) -> Self {
        Self { category, debt }
    }

    pub(crate) fn category(&self) -> &'static str {
        self.category
    }

    pub(crate) fn debt(&self) -> &'static str {
        self.debt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebtInventory {
    pub(crate) suite: &'static str,
    pub(crate) items: Vec<DebtItem>,
}

impl DebtInventory {
    pub(crate) fn new(suite: &'static str, items: Vec<DebtItem>) -> Self {
        Self { suite, items }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn items(&self) -> &[DebtItem] {
        &self.items
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResidualDebtReport {
    pub(crate) suite: &'static str,
    pub(crate) items: Vec<DebtItem>,
}

impl ResidualDebtReport {
    pub(crate) fn new(suite: &'static str, items: Vec<DebtItem>) -> Self {
        Self { suite, items }
    }

    pub(crate) fn suite(&self) -> &'static str {
        self.suite
    }

    pub(crate) fn items(&self) -> &[DebtItem] {
        &self.items
    }
}