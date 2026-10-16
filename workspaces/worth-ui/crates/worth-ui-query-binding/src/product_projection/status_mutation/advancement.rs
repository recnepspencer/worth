//! One fresh-authorized native handoff, disclosed current delivery, then actual close.
use super::super::application_authentication::authenticate;
use super::native::Step;
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationRequest, WorthQueryApplicationRequestExt,
};

pub(super) type Live<'a> =
    WorthQueryApplicationLiveSubscription<'a, WorthUiApplicationSchema, WorthUiStatusQueryRequest>;

impl WorthUiStatusMutation {
    /// Advances once. Progress and every refusal return this same original continuation.
    /// History work is a caller ceiling, additionally bounded by native request resources.
    pub fn advance(
        mut self,
        scope: &WorthQueryRequestScope,
        maximum_history_work: NonZeroUsize,
    ) -> Result<WorthUiStatusMutationResult, Self> {
        let principal = match authenticate(self.application.installed_schema(), scope) {
            Ok(principal) => principal,
            Err(denial) => return Err(self.refused(WorthUiStatusMutationStop::Setup(denial))),
        };
        let request = self.application.request(&principal, scope);
        let result =
            self.state
                .advance_in_request(&request, &self.application, maximum_history_work);
        match result {
            Ok(result) => Ok(result),
            Err(denial) => Err(self.refused(denial)),
        }
    }
}
impl MutationState {
    pub(super) fn advance_in_request(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
        application: &WorthUiStatusApplication,
        history: NonZeroUsize,
    ) -> Result<WorthUiStatusMutationResult, WorthUiStatusMutationStop> {
        self.prepare_in_request(request, application)?;
        // Open before executing so the actual committed cause is observed. Every retry
        // opens a genuine observer and must disclose the original retained commit again.
        let mut live = request
            .query(WorthUiStatusQueryRequest::new("platform.pulse.status"))
            .subscribe(WorthQueryApplicationLiveLimits::bounded(4, 1, 64))
            .map_err(|denial| {
                WorthUiStatusMutationStop::Setup(WorthUiStatusOwnerError::LiveOpen(format!(
                    "{denial:?}"
                )))
            })?;
        let result = self.advance_with_live(request, &mut live, history);
        let close = live.close();
        match result {
            Ok(WorthUiStatusMutationResult::Source(publication)) => Ok(
                WorthUiStatusMutationResult::Source(publication.with_live_close(close)),
            ),
            Ok(WorthUiStatusMutationResult::Action(WorthUiStatusActionOutcome::Committed(
                execution,
            ))) => {
                let (publication, evidence) = execution.into_parts();
                Ok(WorthUiStatusMutationResult::Action(WorthUiStatusActionOutcome::Committed(
                    super::super::application_action_owner::WorthUiStatusActionExecution::from_parts(publication.with_live_close(close), evidence)
                )))
            }
            Ok(WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                    active_revision,
                    submitted_revision,
                    ..
                },
            )) => Ok(WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                    active_revision,
                    submitted_revision,
                    live_close: Some(close),
                },
            )),
            Err(denial) => {
                self.live_close = Some(close);
                Err(denial)
            }
        }
    }
    pub(super) fn advance_with_live(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
        live: &mut Live<'_>,
        history: NonZeroUsize,
    ) -> Result<WorthUiStatusMutationResult, WorthUiStatusMutationStop> {
        let already_completed = self
            .native
            .as_ref()
            .and_then(NativeCustody::retained)
            .is_some();
        let (native, step) = self
            .native
            .take()
            .expect("native owner retained")
            .advance(request, history);
        self.native = Some(native);
        match step? {
            Step::Progress => return Err(WorthUiStatusMutationStop::Progress),
            Step::Terminal => return self.finish_terminal(request),
            Step::Ready => {}
        }
        if already_completed {
            self.disclose_current_retained(request)?;
        } else {
            self.deliver(request, live)?;
        }
        self.close_native(request)
            .map_err(WorthUiStatusMutationStop::Close)?;
        let publication = self
            .publication
            .take()
            .expect("actual disclosed publication retained");
        Ok(match std::mem::replace(&mut self.context, MutationContext::Source) {
            MutationContext::Source => WorthUiStatusMutationResult::Source(publication),
            MutationContext::Action { source_revision, status } => WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::Committed(super::super::application_action_owner::WorthUiStatusActionExecution::from_publication(publication, source_revision, status))
            ),
        })
    }
    fn finish_terminal(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
    ) -> Result<WorthUiStatusMutationResult, WorthUiStatusMutationStop> {
        if !self
            .native
            .as_ref()
            .expect("terminal owner retained")
            .revision_mismatch()
        {
            return Err(WorthUiStatusMutationStop::Terminal);
        }
        self.close_native(request)
            .map_err(WorthUiStatusMutationStop::Close)?;
        let MutationContext::Action {
            source_revision, ..
        } = &self.context
        else {
            unreachable!("only actual action denial is classified");
        };
        Ok(WorthUiStatusMutationResult::Action(
            WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                active_revision: self
                    .active_revision
                    .expect("initiating source revision retained"),
                submitted_revision: *source_revision,
                live_close: None,
            },
        ))
    }
}
