use worth_query_installation::facade::{
    WorthQueryInstalledGraphObligationEffectPosture, WorthQueryInstalledGraphObligationSet,
    WorthQueryInstalledGraphObligationSubjectKind,
    WorthQueryRetainedApplicationQueryGraphObligations,
};

use super::selected_set::SelectedInstalledGraphObligations;

use super::{
    WorthQueryGraphObligationSelectionCounters, WorthQueryGraphObligationSelectionDenial,
    WorthQueryGraphObligationSelectionDenialKind as DenialKind, WorthQueryGraphWorkIntent,
    WorthQueryGraphWorkIntentKind, WorthQuerySelectedGraphObligations,
};

pub fn select_installed_graph_obligations(
    installed: WorthQueryInstalledGraphObligationSet,
    intent: WorthQueryGraphWorkIntent,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionDenial> {
    select_core(SelectedInstalledGraphObligations::Owned(installed), intent)
}

pub fn select_shared_application_query_graph_obligations(
    installed: WorthQueryRetainedApplicationQueryGraphObligations,
    intent: WorthQueryGraphWorkIntent,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionDenial> {
    select_core(SelectedInstalledGraphObligations::Shared(installed), intent)
}

pub(super) fn select_core(
    installed: SelectedInstalledGraphObligations,
    intent: WorthQueryGraphWorkIntent,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionDenial> {
    let set = installed.installed_set();
    let mut counters = WorthQueryGraphObligationSelectionCounters::default();
    counters.checked_subject();
    validate_subject(set, intent)
        .map_err(|kind| WorthQueryGraphObligationSelectionDenial::new(kind, set.subject_name()))?;
    let has_mutation = set.rows().iter().any(|row| {
        matches!(
            row.effect_posture(),
            WorthQueryInstalledGraphObligationEffectPosture::Mutating
                | WorthQueryInstalledGraphObligationEffectPosture::Invariant
        )
    });
    validate_effect_posture(has_mutation, intent)
        .map_err(|kind| WorthQueryGraphObligationSelectionDenial::new(kind, set.subject_name()))?;
    for _ in set.rows() {
        counters.examined_row();
        counters.selected_row();
    }
    Ok(WorthQuerySelectedGraphObligations::seal(
        installed, intent, counters,
    ))
}

fn validate_subject(
    installed: &WorthQueryInstalledGraphObligationSet,
    intent: WorthQueryGraphWorkIntent,
) -> Result<(), DenialKind> {
    let expected = match intent.kind() {
        WorthQueryGraphWorkIntentKind::ApplicationQueryRead => {
            WorthQueryInstalledGraphObligationSubjectKind::ApplicationQuery
        }
        WorthQueryGraphWorkIntentKind::ApplicationOperationRead
        | WorthQueryGraphWorkIntentKind::ApplicationOperationMutation => {
            WorthQueryInstalledGraphObligationSubjectKind::ApplicationOperation
        }
    };
    (installed.subject_kind() == expected)
        .then_some(())
        .ok_or(DenialKind::SubjectKindMismatch)
}

fn validate_effect_posture(
    has_mutation: bool,
    intent: WorthQueryGraphWorkIntent,
) -> Result<(), DenialKind> {
    match intent.kind() {
        WorthQueryGraphWorkIntentKind::ApplicationOperationMutation if !has_mutation => {
            Err(DenialKind::MutationAuthorityRequired)
        }
        WorthQueryGraphWorkIntentKind::ApplicationQueryRead
        | WorthQueryGraphWorkIntentKind::ApplicationOperationRead
            if has_mutation =>
        {
            Err(DenialKind::ReadOnlyIntentCannotSelectMutation)
        }
        _ => Ok(()),
    }
}
