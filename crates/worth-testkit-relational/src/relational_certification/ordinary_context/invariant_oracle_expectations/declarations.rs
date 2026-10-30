pub use {crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, crate::relational_certification::ordinary_context::world::supply_chain::program::CompiledSupplyChainProgram, crate::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation, crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch, crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState};

pub fn expected_supply_chain_branch(
    program: &CompiledSupplyChainProgram,
    branch: BranchLabel,
    delta: Option<DeltaId>,
) -> ExpectedSupplyChainObservation {
    let parent = crate::relational_certification::ordinary_context::world::supply_chain::oracle::ancestry::OracleBranch::genesis(crate::relational_certification::ordinary_context::world::supply_chain::oracle::state::OracleState::from_definition(program.definition()));
    let child = if branch == crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating {
        parent
    } else {
        parent
            .fork(branch, crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::Operating)
            .expect("the Phase-5 branch has the declared operating parent")
    };
    let child = match delta {
        Some(delta) => child
            .apply(delta)
            .expect("the independent oracle accepts the named Phase-5 delta"),
        None => child,
    };
    let mut expected = crate::relational_certification::ordinary_context::world::supply_chain::expected_observation::ExpectedSupplyChainObservation::from_branch(&child);
    // Production exposes owner lineage, not the certification oracle's
    // accepted-delta journal. The semantic state remains oracle-derived.
    expected.ancestry.accepted.clear();
    expected.ancestry.history.clear();
    expected
}
