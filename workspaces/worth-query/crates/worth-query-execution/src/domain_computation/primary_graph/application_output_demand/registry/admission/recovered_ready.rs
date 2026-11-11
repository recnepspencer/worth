//! A live original receipt joined to retained output keeps its exact obligation.
use super::super::{performed_join::PreparedPerformedJoin, DemandRegistryState};
use super::*;

pub(super) fn prepare(
    state: &DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,
    selected: &WorthQueryOutputDemandKey,
    expected: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    kind: DemandAdmissionKind,
) -> Result<Option<PreparedPerformedJoin>, WorthQueryOutputDemandDenial> {
    match kind {
        DemandAdmissionKind::Recovery => {}
        DemandAdmissionKind::Ordinary | DemandAdmissionKind::Required => return Ok(None),
    }
    let Some(commit) = expected else {
        return Ok(None);
    };
    // A retained join may already have consumed this source. Reusing that
    // admission must never consume its original obligation a second time.
    if !state
        .source_custody
        .get(commit)
        .is_some_and(|custody| custody.available(&requested.source, scope))
    {
        return Ok(None);
    }
    state
        .prepare_performed_join(requested, selected, commit, scope, occurrence)
        .map(Some)
}
