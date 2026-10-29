use super::super::type_shapes::{DebtItem, ResidualDebtReport};

pub fn residual_debt_report() -> ResidualDebtReport {
    ResidualDebtReport::new(
        "lowering_and_execution_readiness_boundary",
        vec![DebtItem::new(
            "representative_scope",
            "Milestone 5 closes the canonical lowered-versus-ready boundary with representative runtime-admission and executed-state hooks, but later milestones still add multi-artifact composition pressure and cross-crate migration proof on top of this substrate.",
        )],
    )
}
