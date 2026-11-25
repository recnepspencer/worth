//! Phase-isolated admission selection and record installation helpers.
use super::*;

#[inline(never)]
pub(super) fn select_recovery_key(
    state: &super::super::DemandRegistryState,
    requested_key: &WorthQueryOutputDemandKey,
    requested_source: &super::super::SourceEpoch,
    source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
    let expected = expected_source_commit.ok_or_else(|| {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            "recovery requires the exact source publication",
        )
    })?;
    let custody = state.source_custody.get(expected).ok_or_else(|| {
        WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            "the requested publication has no retained output custody",
        )
    })?;
    if let Some(retired) = &custody.retired {
        return Err(retired.clone());
    }
    if let Some(retired) = custody.source_denial(requested_source) {
        return Err(retired);
    }
    let same_occurrence = custody.occurrence == product_occurrence;
    let retained_key = newest_semantic_key(state, requested_key, |record| {
        (record.is_required()
            || record.has_cached_ready()
            || record.performed_source.as_ref().is_some_and(|source| {
                source.observation.selected_commit() == expected
                    && source.output_source_identity.as_ref() == Some(requested_source)
            }))
            && record.source_commits.contains(expected)
            && record.product_occurrence == product_occurrence
            && record.source_scope == Some(source_scope)
    })?;

    if let Some(DemandState::Failed(denial)) = retained_key
        .as_ref()
        .and_then(|key| state.records.get(key))
        .map(|record| &record.state)
    {
        return Err(denial.clone());
    }
    let retained_record = retained_key.is_some();
    let available = custody.available(requested_source, source_scope);
    if !same_occurrence || (!retained_record && !available) {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            "exact publication, source scope, and root identity are not retained",
        ));
    }
    Ok(retained_key)
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn select_existing_key(
    state: &super::super::DemandRegistryState,
    requested_key: &WorthQueryOutputDemandKey,
    selected_output: Option<&SelectedOutputAdmission<'_>>,
    source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    admission_kind: DemandAdmissionKind,
    expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
    successor_of: Option<OutputRefreshPredecessor<'_>>,
    matching_delivery: Option<WorthQueryOutputDemandKey>,
) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
    let stable_refresh = matches!(successor_of, Some(OutputRefreshPredecessor::Stable { .. }));
    let exact_retained_row =
        if let Some(selected) = selected_output.filter(|_| successor_of.is_none()) {
            // An explicit accepted-authority refresh already rejected its Ready.
            // A retained selector hint cannot rejoin that stale source address.
            super::super::refreshed_rejoin::newest_matching_row(
                &state.records,
                requested_key,
                |key, record| {
                    record.product_occurrence == product_occurrence
                        && record.source_scope == Some(source_scope)
                        && (admission_kind != DemandAdmissionKind::Recovery
                            || expected_source_commit
                                .is_some_and(|expected| record.source_commits.contains(expected)))
                        && selected.matches_retained(requested_key, key, record)
                },
            )
        } else {
            None
        };

    let matching_semantic_source = newest_semantic_key(state, requested_key, |record| {
        accepts_semantic_join(record)
            && (!stable_refresh
                || matches!(&record.state, DemandState::Output(output)
                if matches!(output.checkpoint.as_ref(), Some(super::super::WorthQueryOutputCheckpoint::Ready(completion))
                    if matches!(completion.authority, super::super::WorthQueryAcceptedOutputAuthority::Stable(_)))))
    })?;
    let retained_key = if admission_kind == DemandAdmissionKind::Recovery {
        select_recovery_key(
            state,
            requested_key,
            &requested_key.source,
            source_scope,
            product_occurrence,
            expected_source_commit,
        )?
    } else {
        None
    };

    // A Stable refresh reopens the held semantic row. Its frozen source
    // meaning stays installed; every retained read selects the current
    // Product without releasing the row's settlements or upstream claims.
    if retained_key.is_some() {
        Ok(retained_key)
    } else if let Some(key) = exact_retained_row {
        Ok(Some(key))
    } else if state.records.contains_key(requested_key) {
        Ok(Some(requested_key.clone()))
    } else if let Some(key) = matching_semantic_source {
        Ok(Some(key))
    } else if let Some(key) = matching_delivery {
        Ok(Some(key))
    } else if successor_of.is_none() && admission_kind != DemandAdmissionKind::Recovery {
        // Schedule the real cached predecessor before replacing it. Ready
        // certification still decides currentness; its existing refresh
        // owns the accepted authority and consumed-output handoff.
        cached_ready::predecessor(state, requested_key, source_scope, product_occurrence)
    } else {
        Ok(None)
    }
}
