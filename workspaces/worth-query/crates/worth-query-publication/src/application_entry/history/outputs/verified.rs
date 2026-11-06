//! A verified root shares the existing child traversal and cleanup owners.
use super::{retained::*, tree::TreeCompletion, *};
use crate::application_entry::mutation::program_output_continuation::{
    ProgramOutputContinuation, ProgramOutputContinuationFactory, ProgramOutputContinuationProgress,
    ProgramOutputHistoryFrame,
};
use crate::application_entry::mutation::ProgramOutputRecord;
use worth_query_execution::publication_boundary::{
    discovery_currentness::WorthQueryRetainedQueryDependencies,
    document_history::{WorthQueryHistoryOutputAuthority, WorthQueryVerifiedHistoryParent},
};

pub(super) struct VerifiedTree<
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
> {
    parent: WorthQueryVerifiedHistoryParent<S, P, D>,
    pub(super) continuation: Box<dyn ProgramOutputContinuation<S, P>>,
    closed: Option<(
        Vec<ProgramOutputRecord>,
        Option<WorthQueryRetainedQueryDependencies>,
    )>,
}
impl<S, P, R> WorthQueryHistoryWithOutputs<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        worth_query_declaration::facade::application_query::ApplicationQueryIntent<
            S,
            Binding = Source<S, R>,
        >,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    pub(super) fn try_start_verified(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        state: &mut SlotState<S, P, R>,
    ) -> Result<bool, WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        let Some(parent) = self
            .application
            .verify_history_root_parent::<R>(
                phase,
                &program_publication_access(),
                authority,
                request.principal,
            )
            .map_err(Denial::GraphQualification)?
        else {
            return Ok(false);
        };
        let continuation = R::Dependents::start_verified(
            &self.application,
            &self.demand,
            &parent,
            request,
            self.controls,
        )
        .map_err(Denial::Preparation)?;
        state.tree = Some(HistoryOutputTraversal::Verified(VerifiedTree {
            parent,
            continuation,
            closed: None,
        }));
        Ok(true)
    }

    pub(super) fn progress_verified(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        frame: ProgramOutputHistoryFrame<'_, '_, '_>,
        selecting: bool,
        state: &mut SlotState<S, P, R>,
    ) -> Result<Option<TreeCompletion>, WorthQueryHistoryOutputDenial> {
        use WorthQueryHistoryOutputDenial as Denial;
        // Reauthorize the root's actual source at C even after descendants close.
        let fresh = self
            .application
            .verify_history_root_parent::<R>(
                phase,
                &program_publication_access(),
                frame.authority,
                request.principal,
            )
            .map_err(Denial::GraphQualification)?;
        if fresh.is_none() {
            let parent = &state
                .tree
                .as_ref()
                .and_then(HistoryOutputTraversal::verified)
                .expect("retained verified root")
                .parent;
            return Err(Denial::GraphQualification(
                WorthQueryHistoryOutputQualificationDenial::ParentEvidenceUnavailable {
                    source: parent.source(),
                    family: parent.family(),
                },
            ));
        }
        let tree = state
            .tree
            .as_mut()
            .and_then(HistoryOutputTraversal::verified_mut)
            .expect("the verified root remains owned");
        if selecting || tree.closed.is_some() {
            tree.continuation
                .authorize_history(phase, &self.application, request, frame)
                .map_err(Denial::Preparation)?;
        } else {
            match tree
                .continuation
                .advance(phase, &self.application, request, Some(frame))
            {
                Ok(
                    ProgramOutputContinuationProgress::Pending
                    | ProgramOutputContinuationProgress::Progressed,
                ) => return Ok(None),
                Ok(ProgramOutputContinuationProgress::Settled {
                    outputs,
                    dependencies,
                    ..
                }) => tree.closed = Some((outputs, dependencies)),
                Err(mut cause) => {
                    state.read_cleanup = cause.take_read_cleanup();
                    state.managed_cleanup = cause
                        .take_managed_cleanup()
                        .map(|cleanup| cleanup.into_parts());
                    state.terminal = state.read_cleanup.is_some() || state.managed_cleanup.is_some()
                        || cause.recovery_posture() == crate::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal;
                    return Err(Denial::Preparation(cause));
                }
            }
        }
        if selecting {
            return Ok(None);
        }
        tree.continuation
            .close_read_request(phase, &self.application, frame.branch)
            .map_err(Denial::Close)?;
        if state.forest_tree.is_none() {
            let (outputs, dependencies) =
                tree.closed.as_mut().expect("the discovered set is closed");
            let retained = self
                .application
                .retain_verified_history_output_tree::<R>(
                    phase,
                    &program_publication_access(),
                    &tree.parent,
                    frame.authority,
                    outputs.iter().map(|output| output.retain_settlement()),
                    dependencies,
                )
                .map_err(Denial::GraphQualification)?;
            state.forest_tree = Some(retained);
        }
        Ok(Some(TreeCompletion::Forest(std::sync::Arc::clone(
            state.forest_tree.as_ref().expect("retained verified tree"),
        ))))
    }
}
