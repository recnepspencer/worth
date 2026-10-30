pub(crate) use {
    crate::shared_support::type_shapes::debt::DebtItem,
    crate::shared_support::type_shapes::debt::ResidualDebtReport,
};

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::type_shapes::debt::ResidualDebtReport::new(
        "transition_outcome_algebra",
        vec![crate::shared_support::type_shapes::debt::DebtItem::new(
            "representative_scope",
            "Milestone 4 closes the canonical transition substrate with representative recipe lanes and explicit category divergence, but later milestones still supply lowering/readiness-specific and multi-artifact domain pressure on top of this shared grammar.",
        )],
    )
}
