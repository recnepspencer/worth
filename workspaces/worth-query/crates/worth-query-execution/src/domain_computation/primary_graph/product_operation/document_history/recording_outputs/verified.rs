//! Actual discovered descendants retain a verified parent without a root settlement.
use super::*;
use crate::domain_computation::primary_graph::{
    application_installation::WorthQueryProgramApplicationRuntime,
    WorthQueryApplicationOutputDemand, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryVerifiedHistoryParent,
};
use worth_query_declaration::facade::application_program::ApplicationConnectionShape;
type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
type Demand<S, R> = <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn retain_verified_recording_output_tree<R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
        parent: &WorthQueryVerifiedHistoryParent<S, P, Demand<S, R>>,
        authority: &crate::domain_computation::primary_graph::WorthQueryHistoryOutputAuthority<
            '_,
            '_,
        >,
        dependents: impl ExactSizeIterator<Item = Arc<WorthQueryOutputDemandSettlement>>,
        dependencies: &mut Option<WorthQueryRetainedQueryDependencies>,
    ) -> Result<
        Arc<WorthQueryRetainedRecordingOutputTree>,
        WorthQueryRecordingOutputQualificationDenial,
    >
    where
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        Demand<S, R>: WorthQueryApplicationOutputDemand<S>,
    {
        let profile = origin
            .profile
            .tree_profiles()
            .get(index)
            .ok_or_else(foreign)?;
        if !profile.matches_recording_root::<S, P, R>(self)
            || !profile.same_custody(authority.profile())
            || authority.recording_model() != Some(profile.recording_model().entity())
            || !Arc::ptr_eq(origin.observation(), authority.original())
        {
            return Err(foreign().into());
        }
        let tree = self
            .retain_verified_history_output_tree::<R>(
                phase,
                access,
                parent,
                authority,
                dependents,
                dependencies,
            )
            .map_err(WorthQueryRecordingOutputQualificationDenial::Verified)?;
        let execution = phase
            .checked_execution_request_for(self.runtime().product_runtime())
            .map_err(stopped)?;
        let memory = memory::reserve(
            &execution,
            execution.has_accounting(),
            (std::mem::size_of::<WorthQueryRetainedRecordingOutputTree>()
                + 2 * std::mem::size_of::<usize>()) as u64,
        )
        .map_err(resource)?;
        Ok(Arc::new(WorthQueryRetainedRecordingOutputTree {
            origin: Arc::clone(origin),
            index,
            completion: RecordingTreeCompletion::VerifiedParent(tree),
            _verification_memory: memory,
        }))
    }
}
