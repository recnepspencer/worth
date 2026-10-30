pub(crate) use {
    crate::shared_support::type_shapes::debt::DebtItem,
    crate::shared_support::type_shapes::debt::ResidualDebtReport,
};

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::type_shapes::debt::ResidualDebtReport::new(
        "static_fork_join_and_composition_family",
        vec![crate::shared_support::type_shapes::debt::DebtItem::new(
            "representative_scope",
            "Milestone 6 closes canonical fixed-arity fork/join progression and deterministic same-family lowering for representative static lanes, but broader N-ary composition pressure and cross-crate migration proof remain explicit Milestone 7 work.",
        )],
    )
}
