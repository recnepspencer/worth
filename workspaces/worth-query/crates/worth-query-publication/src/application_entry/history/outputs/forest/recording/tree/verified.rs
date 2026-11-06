//! Recording discovers descendants from the owner's verified parent, never a settlement.
use super::*;
use crate::application_entry::history::outputs::verified::VerifiedTree;
use crate::application_entry::mutation::program_output_continuation::{
    ProgramOutputContinuationProgress, ProgramOutputHistoryFrame,
};
impl<S, P, R> OriginalRecordingTree<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, R>: Clone,
    Value<S, R>: WorthQueryApplicationProjection<S, RootQuery<S, R>> + Clone,
    <Source<S, R> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, R>>,
    <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    R::Dependents: ProgramOutputContinuationFactory<S, P, Demand<S, R>>,
{
    pub(super) fn try_start_verified(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
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
        self.tree = Some(HistoryOutputTraversal::Verified(VerifiedTree {
            parent,
            continuation,
            closed: None,
        }));
        Ok(true)
    }
    pub(super) fn progress_verified(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    ) -> Result<Option<Arc<WorthQueryRetainedRecordingOutputTree>>, WorthQueryHistoryOutputDenial>
    {
        use WorthQueryHistoryOutputDenial as Denial;
        let fresh = self
            .application
            .verify_history_root_parent::<R>(
                phase,
                &program_publication_access(),
                authority,
                request.principal,
            )
            .map_err(Denial::GraphQualification)?;
        let tree = self
            .tree
            .as_mut()
            .and_then(HistoryOutputTraversal::verified_mut)
            .expect("verified recording parent");
        if fresh.is_none() {
            return Err(Denial::GraphQualification(
                WorthQueryHistoryOutputQualificationDenial::ParentEvidenceUnavailable {
                    source: tree.parent.source(),
                    family: tree.parent.family(),
                },
            ));
        }
        let frame = ProgramOutputHistoryFrame {
            authority,
            branch: origin.candidate(),
        };
        if tree.closed.is_some() {
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
                    self.read_cleanup = cause.take_read_cleanup();
                    self.managed_cleanup = cause
                        .take_managed_cleanup()
                        .map(|cleanup| cleanup.into_parts());
                    self.terminal = self.read_cleanup.is_some() || self.managed_cleanup.is_some()
                        || cause.recovery_posture() == crate::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal;
                    return Err(Denial::Preparation(cause));
                }
            }
        }
        tree.continuation
            .close_read_request(phase, &self.application, origin.candidate())
            .map_err(Denial::Close)?;
        let (outputs, dependencies) = tree.closed.as_mut().expect("closed actual discovery");
        let retained = self
            .application
            .retain_verified_recording_output_tree::<R>(
                phase,
                &program_publication_access(),
                origin,
                index,
                &tree.parent,
                authority,
                outputs.iter().map(|output| output.retain_settlement()),
                dependencies,
            )
            .map_err(Denial::RecordingQualification)?;
        self.retained = Some(Arc::clone(&retained));
        Ok(Some(retained))
    }
}
