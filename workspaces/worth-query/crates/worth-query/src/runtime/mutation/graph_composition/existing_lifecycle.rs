use super::denial::{graph_composition_error, WorthQueryGraphCompositionDenialKind};
use crate::runtime::{
    WorthQueryContinuityMutationOutcomeClass, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryNamingMutationFamily, WorthQueryRuntimeError, WorthQueryWriteCommand,
};

pub(super) fn require_retarget_intent(
    command: &WorthQueryWriteCommand,
    declared_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> Result<(), WorthQueryRuntimeError> {
    if command.continuity_intent().is_some_and(|intent| {
        matches!(
            intent.outcome_class(),
            WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors
                | WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
        )
    }) {
        return Err(graph_composition_error(
            WorthQueryGraphCompositionDenialKind::ExistingTargetIdentityPreservationUnavailable,
            None,
            declared_collection.cloned(),
            "graph retarget lanes preserve one continuing target identity; split or merge continuity requires supersession_with_lineage instead",
        )?);
    }
    let has_naming_rebind = command
        .naming_intent()
        .is_some_and(|intent| intent.family() == WorthQueryNamingMutationFamily::RebindTarget);
    let has_continuity_rebind = command.continuity_intent().is_some_and(|intent| {
        intent.outcome_class()
            == WorthQueryContinuityMutationOutcomeClass::ContinuesAsSingleSuccessor
    });
    if has_naming_rebind || has_continuity_rebind {
        return Ok(());
    }
    Err(graph_composition_error(
        WorthQueryGraphCompositionDenialKind::ExistingTargetRetargetUnsupported,
        None,
        declared_collection.cloned(),
        "graph retarget lanes require naming_rebind_target(...) or continuity_rebind_existing_target(...) on the update component",
    )?)
}

pub(super) fn require_supersession_intent(
    command: &WorthQueryWriteCommand,
    declared_collection: Option<&WorthQueryMutationTargetCollectionIdentity>,
) -> Result<(), WorthQueryRuntimeError> {
    let has_supersession_continuity = command.continuity_intent().is_some_and(|intent| {
        matches!(
            intent.outcome_class(),
            WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors
                | WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
        )
    });
    if has_supersession_continuity {
        return Ok(());
    }
    Err(graph_composition_error(
        WorthQueryGraphCompositionDenialKind::ExistingTargetSupersessionUnsupported,
        None,
        declared_collection.cloned(),
        "graph supersession lanes require continuity_split_successors(...) or continuity_rebind_merge_successor(...) on the update component",
    )?)
}
