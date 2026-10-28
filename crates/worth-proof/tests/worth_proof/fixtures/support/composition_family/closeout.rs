use super::super::type_shapes::{DebtItem, ResidualDebtReport};

pub fn residual_debt_report() -> ResidualDebtReport {
    ResidualDebtReport::new(
        "static_fork_join_and_composition_family",
        vec![DebtItem::new(
            "representative_scope",
            "Milestone 6 closes canonical fixed-arity fork/join progression and deterministic same-family lowering for representative static lanes, but broader N-ary composition pressure and cross-crate migration proof remain explicit Milestone 7 work.",
        )],
    )
}
