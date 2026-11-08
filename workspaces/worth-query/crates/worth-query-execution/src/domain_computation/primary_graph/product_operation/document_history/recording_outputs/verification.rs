//! Distinct current or outside-domain evidence; neither is an SDK settlement.
use super::*;
#[derive(PartialEq, Eq)]
pub(super) enum TreeVerification {
    CurrentLeaf,
    OutsideDomain,
    SourceNotLive,
    RequiresOriginal,
}
impl WorthQueryHistoryRecordingOutputOrigin {
    pub(super) fn verify_tree_at<S: ApplicationSchema>(
        &self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        index: usize,
        data: &Arc<WorthQueryApplicationReadObservation>,
    ) -> Result<TreeVerification, WorthQueryRecordingOutputQualificationDenial> {
        let profile = self
            .profile
            .tree_profiles()
            .get(index)
            .ok_or_else(foreign)?;
        let selected = application
            .select_application_read_observation(phase, data)
            .map_err(WorthQueryOriginalReadOutputDenial::Observation)?;
        if data.branch_incarnation() != self.candidate.occurrence() {
            return Err(foreign().into());
        }
        let execution = phase
            .checked_execution_request_for(&application.runtime.product_runtime)
            .map_err(stopped)?;
        let graph = &application.runtime.primary_provider.graph;
        graph.with_runtime(&execution, |native| {
            let owner = match native
                .observe_model_owner(
                    &execution,
                    selected.product().relational_basis(),
                    graph.layout.installed_history_ownership(),
                    profile.source_intent().source_subject(),
                )
            {
                Ok(owner) => owner,
                Err(worth_relational::facade::branch::RelationalModelOwnershipDenial::EntityUnavailable(subject))
                    if subject == profile.source_intent().source_subject() => return Ok(TreeVerification::SourceNotLive),
                Err(cause) => return Err(WorthQueryRecordingOutputQualificationDenial::Ownership(Box::new(cause))),
            };
            if owner != Some(profile.recording_model().entity()) {
                return Ok(TreeVerification::OutsideDomain);
            }
            if !profile.recording_is_leaf() {
                return Ok(TreeVerification::RequiresOriginal);
            }
            let current=selected.current_owned_route_is_current(&execution,native,profile.recording_model().entity(),
                profile.source_intent().source_subject(),profile.recording_family(),profile.source_intent().source_partition())
                .map_err(|cause|match cause {
                    crate::domain_computation::primary_graph::application_output_demand::CurrentOwnedRouteDenial::Ownership(cause)=>WorthQueryRecordingOutputQualificationDenial::Ownership(Box::new(cause)),
                    crate::domain_computation::primary_graph::application_output_demand::CurrentOwnedRouteDenial::Output(cause)=>WorthQueryOriginalReadOutputDenial::Output(cause).into(),
                })?;
            Ok(if current {TreeVerification::CurrentLeaf} else {TreeVerification::RequiresOriginal})
        })
    }
}
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramApplicationRuntime;
impl<S, P> WorthQueryProgramApplicationRuntime<S, P>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
{
    /// Current leaves and sources outside the document need no original root.
    /// Live non-leaf roots still run actual installed discovery.
    pub fn verify_recording_output_tree<R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
        data: &Arc<WorthQueryApplicationReadObservation>,
    ) -> Result<
        Option<Arc<WorthQueryRetainedRecordingOutputTree>>,
        WorthQueryRecordingOutputQualificationDenial,
    > {
        let profile = origin
            .profile
            .tree_profiles()
            .get(index)
            .ok_or_else(foreign)?;
        if !profile.matches_recording_root::<S, P, R>(self) {
            return Err(foreign().into());
        }
        let current = self
            .runtime()
            .on_branch(origin.candidate)
            .select(phase)
            .map_err(WorthQueryOriginalReadOutputDenial::Observation)?;
        if current.product().observation() != data.product_observation() {
            return Err(foreign().into());
        }
        let completion = match origin.verify_tree_at(self.runtime(), phase, index, data)? {
            TreeVerification::CurrentLeaf => RecordingTreeCompletion::VerifiedLeaf,
            TreeVerification::OutsideDomain => RecordingTreeCompletion::SourceOutsideDomain,
            TreeVerification::SourceNotLive => RecordingTreeCompletion::SourceNotLive,
            TreeVerification::RequiresOriginal => return Ok(None),
        };
        let execution = phase
            .checked_execution_request_for(&self.runtime().runtime.product_runtime)
            .map_err(stopped)?;
        let memory = memory::reserve(
            &execution,
            execution.has_accounting(),
            (std::mem::size_of::<WorthQueryRetainedRecordingOutputTree>()
                + 2 * std::mem::size_of::<usize>()) as u64,
        )
        .map_err(resource)?;
        Ok(Some(Arc::new(WorthQueryRetainedRecordingOutputTree {
            origin: Arc::clone(origin),
            index,
            completion,
            _verification_memory: memory,
        })))
    }
}
