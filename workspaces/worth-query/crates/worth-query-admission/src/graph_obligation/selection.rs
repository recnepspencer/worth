use std::convert::Infallible;

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

#[derive(Debug)]
pub enum WorthQueryGraphObligationSelectionAdmissionStop<Stop> {
    Admission(Stop),
    Selection(WorthQueryGraphObligationSelectionDenial),
    AccountingOverflow,
}

pub fn select_installed_graph_obligations(
    installed: WorthQueryInstalledGraphObligationSet,
    intent: WorthQueryGraphWorkIntent,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionDenial> {
    match select_installed_graph_obligations_admitted(installed, intent, |_, _| {
        Ok::<(), Infallible>(())
    }) {
        Ok(selected) => Ok(selected),
        Err(WorthQueryGraphObligationSelectionAdmissionStop::Selection(denial)) => Err(denial),
        Err(WorthQueryGraphObligationSelectionAdmissionStop::Admission(never)) => match never {},
        Err(WorthQueryGraphObligationSelectionAdmissionStop::AccountingOverflow) => {
            unreachable!("installed graph obligation lengths fit usize and u64")
        }
    }
}

/// Uses the ordinary selector's exact acceptance core. The caller has already
/// admitted retention of the installed set before passing its owned copy.
pub fn select_installed_graph_obligations_admitted<Stop>(
    installed: WorthQueryInstalledGraphObligationSet,
    intent: WorthQueryGraphWorkIntent,
    admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionAdmissionStop<Stop>>
{
    select_core(
        SelectedInstalledGraphObligations::Owned(installed),
        intent,
        admit,
    )
}

pub fn select_shared_application_query_graph_obligations_admitted<Stop>(
    installed: WorthQueryRetainedApplicationQueryGraphObligations,
    intent: WorthQueryGraphWorkIntent,
    admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionAdmissionStop<Stop>>
{
    select_core(
        SelectedInstalledGraphObligations::Shared(installed),
        intent,
        admit,
    )
}

fn select_core<Stop>(
    installed: SelectedInstalledGraphObligations,
    intent: WorthQueryGraphWorkIntent,
    mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionAdmissionStop<Stop>>
{
    let set = installed.installed_set();
    let mut counters = WorthQueryGraphObligationSelectionCounters::default();
    admit(1, 0).map_err(WorthQueryGraphObligationSelectionAdmissionStop::Admission)?;
    counters.checked_subject();
    if let Err(kind) = validate_subject(set, intent) {
        return denied(kind, set, &mut admit);
    }
    let mut has_mutation = false;
    for row in set.rows() {
        admit(1, 0).map_err(WorthQueryGraphObligationSelectionAdmissionStop::Admission)?;
        if matches!(
            row.effect_posture(),
            WorthQueryInstalledGraphObligationEffectPosture::Mutating
                | WorthQueryInstalledGraphObligationEffectPosture::Invariant
        ) {
            has_mutation = true;
            break;
        }
    }
    admit(1, 0).map_err(WorthQueryGraphObligationSelectionAdmissionStop::Admission)?;
    if let Err(kind) = validate_effect_posture(has_mutation, intent) {
        return denied(kind, set, &mut admit);
    }
    for _ in set.rows() {
        admit(2, 0).map_err(WorthQueryGraphObligationSelectionAdmissionStop::Admission)?;
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

fn denied<Stop>(
    kind: DenialKind,
    installed: &WorthQueryInstalledGraphObligationSet,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<WorthQuerySelectedGraphObligations, WorthQueryGraphObligationSelectionAdmissionStop<Stop>>
{
    let bytes = u64::try_from(installed.subject_name().len())
        .map_err(|_| WorthQueryGraphObligationSelectionAdmissionStop::AccountingOverflow)?;
    admit(bytes, bytes).map_err(WorthQueryGraphObligationSelectionAdmissionStop::Admission)?;
    Err(WorthQueryGraphObligationSelectionAdmissionStop::Selection(
        WorthQueryGraphObligationSelectionDenial::new(kind, installed.subject_name()),
    ))
}
