use crate::shared_support::type_shapes::{DebtItem, ResidualDebtReport};

pub fn residual_debt_report() -> ResidualDebtReport {
    ResidualDebtReport::new(
        "pleasant_lane_closeout_debt",
        vec![
            DebtItem::new(
                "codegen",
                "DX hot-path honesty is still size/layout/drop-scoped rather than MIR/ASM-diff certified.",
            ),
            DebtItem::new(
                "adoption",
                "Cross-crate pleasant-lane migration parity remains a Milestone 7 concern and is not certified by the DX-local suite.",
            ),
        ],
    )
}
