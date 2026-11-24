#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityStatus {
    Verified,
    Debt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexityContract {
    pub id: &'static str,
    pub function_path: &'static str,
    pub declared_time_complexity: &'static str,
    pub budget_summary: &'static str,
    pub status: ComplexityStatus,
    pub proof_tests: &'static [&'static str],
}
