pub(crate) use {
    crate::shared_support::type_shapes::debt::DebtItem,
    crate::shared_support::type_shapes::debt::ResidualDebtReport,
};

pub(crate) fn residual_debt_report() -> ResidualDebtReport {
    crate::shared_support::type_shapes::debt::ResidualDebtReport::new(
        "pleasant_lane_closeout_debt",
        vec![
            crate::shared_support::type_shapes::debt::DebtItem::new(
                "codegen",
                "DX hot-path honesty is still size/layout/drop-scoped rather than MIR/ASM-diff certified.",
            ),
            crate::shared_support::type_shapes::debt::DebtItem::new(
                "adoption",
                "Cross-crate pleasant-lane migration parity remains a Milestone 7 concern and is not certified by the DX-local suite.",
            ),
        ],
    )
}
