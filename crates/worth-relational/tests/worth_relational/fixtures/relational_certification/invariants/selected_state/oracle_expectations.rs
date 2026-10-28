pub(crate) use super::world::supply_chain::{
    BranchLabel, CompiledSupplyChainProgram, DeltaId, ExpectedSupplyChainObservation, OracleBranch,
    OracleState,
};

pub(crate) fn expected_supply_chain_branch(
    program: &CompiledSupplyChainProgram,
    branch: BranchLabel,
    delta: Option<DeltaId>,
) -> ExpectedSupplyChainObservation {
    let parent = OracleBranch::genesis(OracleState::from_definition(program.definition()));
    let child = if branch == BranchLabel::Operating {
        parent
    } else {
        parent
            .fork(branch, BranchLabel::Operating)
            .expect("the Phase-5 branch has the declared operating parent")
    };
    let child = match delta {
        Some(delta) => child
            .apply(delta)
            .expect("the independent oracle accepts the named Phase-5 delta"),
        None => child,
    };
    let mut expected = ExpectedSupplyChainObservation::from_branch(&child);
    // Production exposes owner lineage, not the certification oracle's
    // accepted-delta journal. The semantic state remains oracle-derived.
    expected.ancestry.accepted.clear();
    expected.ancestry.history.clear();
    expected
}
