//! Readonly leaf verification never constructs an original root settlement.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryHistoryOutputQualificationDenial as Denial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    mvcc::RequiredOutputPublicationMeaning,
    runtime::{RelationalHistoricalSelectionRead, RelationalRuntime},
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// This answers a borrowed currentness question only. Final qualification
    /// independently checks every skipped route at the derived endpoint.
    pub fn can_verify_selected_history_leaf(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    ) -> Result<bool, Denial> {
        let control = |cause| Denial::Control(Box::new(cause));
        authority
            .validate(phase, self, authority.request())
            .map_err(control)?;
        let result = (|| {
            if !authority.profile().uses_owned_model_graph()
                || !authority.profile().leaf
                || authority.profile().is_output_forest()
            {
                return Ok(false);
            }
            let Some(historical) = authority.selected_graph() else {
                return Ok(false);
            };
            let execution = phase
                .checked_execution_request_for(&self.runtime.product_runtime)
                .map_err(|cause| control(WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
            let selected = self.on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                authority.original().product_observation().lifecycle_incarnation(),
            )).select(phase).map_err(|cause| Denial::Original(crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial::Observation(cause)))?;
            self.runtime
                .primary_provider
                .graph
                .with_runtime(&execution, |native| {
                    let evidence = native
                        .read_truth()
                        .read_historical_selection(
                            &execution,
                            historical,
                            &authority
                                .original()
                                .product_observation()
                                .basis()
                                .relational_basis()
                                .observation(),
                        )
                        .map_err(Denial::SelectionRead)?;
                    profile_is_current(
                        &execution,
                        native,
                        &evidence,
                        &selected,
                        authority.profile(),
                    )
                })
        })();
        authority
            .confirm_read(phase, self, authority.request(), result)
            .map_err(control)?
    }
}

pub(super) fn profile_is_current<S: ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    evidence: &RelationalHistoricalSelectionRead,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    profile: &WorthQueryHistoryOutputProfile,
) -> Result<bool, Denial> {
    if !profile.leaf {
        return Ok(false);
    }
    selected_profile_root_is_current(execution, native, evidence, selected, profile)
}

/// This proves one restored root route only. Its descendants still require
/// actual discovery and complete-set qualification, even when every route is current.
pub(in crate::domain_computation::primary_graph) fn selected_profile_root_is_current<
    S: ApplicationSchema,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    evidence: &RelationalHistoricalSelectionRead,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    profile: &WorthQueryHistoryOutputProfile,
) -> Result<bool, Denial> {
    let mut count = 0;
    let mut current = true;
    for route in evidence
        .selected_output_routes(execution)
        .map_err(Denial::SelectionRead)?
    {
        let route = route.map_err(Denial::SelectionRead)?;
        if matches_profile(execution, route, profile)? {
            count += 1;
            current &= selected.selected_route_is_current(execution, native, route)
                .map_err(|cause| Denial::Original(crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial::Output(cause)))?;
        }
    }
    Ok(count == 1 && current)
}

fn matches_profile(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    route: &RequiredOutputPublicationMeaning,
    profile: &WorthQueryHistoryOutputProfile,
) -> Result<bool, Denial> {
    super::output_coverage::charge(
        execution,
        (route.family.len() + profile.family.len()) as u64 + 8,
    )?;
    Ok(route.family == profile.family
        && route.source_scope == profile.source_intent().source_subject())
}
