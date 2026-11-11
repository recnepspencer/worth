//! Readonly leaf verification never constructs an original root settlement.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryHistoryOutputQualificationDenial as Denial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{RelationalHistoricalSelectionRead, RelationalRuntime};

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
            if authority.selected_graph().is_none() {
                return Ok(false);
            }
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
                    let evidence = authority
                        .selected_output_read(&execution, native)
                        .map_err(Denial::SelectionRead)?;
                    profile_is_current(&execution, native, evidence, &selected, authority.profile())
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
    let route = evidence
        .selected_output_route(
            execution,
            profile.family,
            profile.source_intent().source_subject(),
            worth_foundational::facade::CanonicalDigestId::new(
                profile.source_intent().source_partition(),
            ),
        )
        .map_err(Denial::SelectionRead)?;
    let Some(route) = route else {
        return Ok(false);
    };
    selected.selected_route_is_current(execution, native, route)
        .map_err(|cause| Denial::Original(crate::domain_computation::primary_graph::WorthQueryOriginalReadOutputDenial::Output(cause)))
}

/// Recording verifies its actual current candidate through Native's current facts.
pub(in crate::domain_computation::primary_graph) fn current_recording_route_is_current<
    S: ApplicationSchema,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    native: &RelationalRuntime,
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    model: worth_relational::facade::identity::EntityId,
    source: worth_relational::facade::identity::EntityId,
    family: &str,
    partition: [u8; 32],
) -> Result<bool, crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial> {
    use crate::domain_computation::primary_graph::{
        WorthQueryDocumentHistoryDenial as Document, WorthQueryHistoryOutputReadDenial as Read,
    };
    selected.current_owned_route_is_current(execution,native,model,source,family,partition).map_err(|cause|match cause {
        crate::domain_computation::primary_graph::application_output_demand::CurrentOwnedRouteDenial::Ownership(cause)=>Read::Authority(WorthQueryHistoryNavigationDenial::Document(Document::Ownership(cause))),
        crate::domain_computation::primary_graph::application_output_demand::CurrentOwnedRouteDenial::Output(cause)=>Read::Demand(cause),
    })
}
