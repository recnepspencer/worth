pub use {
    worth_foundational::foundational_merge, worth_foundational::FoundationalMergeScope,
    worth_foundational::FoundationalSelectedAspectLocus,
    worth_foundational::FoundationalSelectedAspectRequestEntry,
    worth_foundational::FoundationalSelectedNodeLocus,
    worth_foundational::FoundationalSelectedScopeLocus,
    worth_foundational::FoundationalSelectedScopeNoOpCause,
    worth_foundational::FoundationalSelectedScopeNoOpEvidence,
};

pub use {
    crate::certification::ordinary_context::transitions::fixtures::branch::branch_id,
    crate::certification::ordinary_context::transitions::fixtures::branch::staged_candidate,
};
pub use {
    crate::certification::ordinary_context::transitions::fixtures::merge::authority_first_merge_candidate,
    crate::certification::ordinary_context::transitions::fixtures::merge::merge_basis,
    crate::certification::ordinary_context::transitions::fixtures::merge::merge_summary,
    crate::certification::ordinary_context::transitions::fixtures::merge::strategy_identity,
};

pub fn selected_node(value: &str) -> FoundationalSelectedNodeLocus {
    worth_foundational::FoundationalSelectedNodeLocus::new(value).expect("selected node")
}

pub fn selected_aspect(node: &str, aspect: &str) -> FoundationalSelectedAspectRequestEntry {
    worth_foundational::FoundationalSelectedAspectRequestEntry::new(
        worth_foundational::FoundationalSelectedNodeLocus::new(node).expect("node"),
        worth_foundational::FoundationalSelectedAspectLocus::new(aspect).expect("aspect"),
    )
}

pub fn no_op_for_node(
    node: &str,
    cause: FoundationalSelectedScopeNoOpCause,
) -> FoundationalSelectedScopeNoOpEvidence {
    worth_foundational::FoundationalSelectedScopeNoOpEvidence::new(
        worth_foundational::FoundationalSelectedScopeLocus::Node(selected_node(node)),
        cause,
    )
}

pub fn no_op_for_aspect(
    node: &str,
    aspect: &str,
    cause: FoundationalSelectedScopeNoOpCause,
) -> FoundationalSelectedScopeNoOpEvidence {
    worth_foundational::FoundationalSelectedScopeNoOpEvidence::new(
        worth_foundational::FoundationalSelectedScopeLocus::Aspect(selected_aspect(node, aspect)),
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
