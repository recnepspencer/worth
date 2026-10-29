pub(crate) use {worth_runtime_world::facade::*, crate::court::ordinary_context::CompositeSupplyChainCourt, crate::court::ordinary_context::relational};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_signal::facade::branch::validate_signal_branch_name};

pub(crate) fn plans(relational: bool, signal: bool, name: &str) -> ProductBranchCreationPlans {
    ProductBranchCreationPlans::new(
        if relational {
            RelationalBranchCreationPlan::ForkExact {
                target: BranchId(name.into()),
            }
        } else {
            RelationalBranchCreationPlan::ReuseExact
        },
        if signal {
            SignalBranchCreationPlan::ForkExact {
                target: validate_signal_branch_name(name).unwrap(),
            }
        } else {
            SignalBranchCreationPlan::ReuseExact
        },
    )
}