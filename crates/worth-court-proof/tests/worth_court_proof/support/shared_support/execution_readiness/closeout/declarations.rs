pub(crate) use {crate::shared_support::type_shapes::debt::DebtItem, crate::shared_support::type_shapes::debt::ResidualDebtReport};

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::type_shapes::debt::ResidualDebtReport::new(
        "lowering_and_execution_readiness_boundary",
        vec![crate::shared_support::type_shapes::debt::DebtItem::new(
            "representative_scope",
            "Milestone 5 closes the canonical lowered-versus-ready boundary with representative runtime-admission and executed-state hooks, but later milestones still add multi-artifact composition pressure and cross-crate migration proof on top of this substrate.",
        )],
    )
}