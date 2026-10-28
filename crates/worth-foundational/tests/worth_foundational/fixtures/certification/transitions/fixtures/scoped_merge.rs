use worth_foundational::{
    foundational_merge, FoundationalMergeScope, FoundationalSelectedAspectLocus,
    FoundationalSelectedAspectRequestEntry, FoundationalSelectedNodeLocus,
    FoundationalSelectedScopeLocus, FoundationalSelectedScopeNoOpCause,
    FoundationalSelectedScopeNoOpEvidence,
};

use super::branch::{branch_id, staged_candidate};
use super::merge::{
    authority_first_merge_candidate, merge_basis, merge_summary, strategy_identity,
};

pub fn selected_node(value: &str) -> FoundationalSelectedNodeLocus {
    FoundationalSelectedNodeLocus::new(value).expect("selected node")
}

pub fn selected_aspect(node: &str, aspect: &str) -> FoundationalSelectedAspectRequestEntry {
    FoundationalSelectedAspectRequestEntry::new(
        FoundationalSelectedNodeLocus::new(node).expect("node"),
        FoundationalSelectedAspectLocus::new(aspect).expect("aspect"),
    )
}

pub fn no_op_for_node(
    node: &str,
    cause: FoundationalSelectedScopeNoOpCause,
) -> FoundationalSelectedScopeNoOpEvidence {
    FoundationalSelectedScopeNoOpEvidence::new(
        FoundationalSelectedScopeLocus::Node(selected_node(node)),
        cause,
    )
}

pub fn no_op_for_aspect(
    node: &str,
    aspect: &str,
    cause: FoundationalSelectedScopeNoOpCause,
) -> FoundationalSelectedScopeNoOpEvidence {
    FoundationalSelectedScopeNoOpEvidence::new(
        FoundationalSelectedScopeLocus::Aspect(selected_aspect(node, aspect)),
        cause,
    )
}

pub fn scoped_candidate(
    scope: FoundationalMergeScope,
) -> worth_foundational::FoundationalMergeCandidate<&'static str> {
    foundational_merge(staged_candidate("mesh-update"))
        .into_target_branch(branch_id("main"))
        .with_intent(worth_foundational::FoundationalMergeIntent::ReconcileIntoTarget)
        .with_structural_summary(merge_summary())
        .with_scope(scope)
        .with_merge_basis(merge_basis("feature/geometry", "main"))
        .with_merge_base_selection_basis(
            authority_first_merge_candidate("mesh-update").merge_base_selection_basis(),
        )
        .under_strategy(strategy_identity())
        .with_strategy_descriptor_digest(
            authority_first_merge_candidate("mesh-update").strategy_descriptor_digest(),
        )
        .with_strategy_contract_basis(
            authority_first_merge_candidate("mesh-update").strategy_contract_basis(),
        )
        .with_strategy_basis(authority_first_merge_candidate("mesh-update").strategy_basis())
        .plan()
        .expect("scoped candidate")
}
