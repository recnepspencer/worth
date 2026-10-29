#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtItem {
    category: &'static str,
    debt: &'static str,
}

impl DebtItem {
    pub fn new(category: &'static str, debt: &'static str) -> Self {
        Self { category, debt }
    }

    pub fn category(&self) -> &'static str {
        self.category
    }

    pub fn debt(&self) -> &'static str {
        self.debt
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebtInventory {
    suite: &'static str,
    items: Vec<DebtItem>,
}

impl DebtInventory {
    pub fn new(suite: &'static str, items: Vec<DebtItem>) -> Self {
        Self { suite, items }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn items(&self) -> &[DebtItem] {
        &self.items
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualDebtReport {
    suite: &'static str,
    items: Vec<DebtItem>,
}

impl ResidualDebtReport {
    pub fn new(suite: &'static str, items: Vec<DebtItem>) -> Self {
        Self { suite, items }
    }

    pub fn suite(&self) -> &'static str {
        self.suite
    }

    pub fn items(&self) -> &[DebtItem] {
        &self.items
    }
}
