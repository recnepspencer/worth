use super::continuation::{PlatformPulseQueryMutation, PlatformPulseQueryMutationPurpose};
use super::error::PlatformPulseQueryActionIndeterminate;
use super::*;
use worth_ui::facade::query_binding::{
    WorthUiScalarProjectionActionPreconditionDenial, WorthUiStatusActionOutcome,
    WorthUiStatusActionRequest,
};
impl PlatformPulseQueryLifecycle {
    pub(crate) fn execute_current_action(
        &mut self,
        status: impl Into<String>,
        idempotency_session: u64,
        idempotency_lineage: u64,
    ) -> Result<PlatformPulseQueryActionOutcome, PlatformPulseQueryLifecycleDenial> {
        let source_revision = self.current_source_revision()?;
        let action = WorthUiStatusActionRequest::new(
            source_revision,
            status,
            idempotency_session,
            idempotency_lineage,
        )
        .map_err(PlatformPulseQueryLifecycleDenial::ActionRequest)?;
        let scope = self.next_request_scope();
        let execution = match self
            .owner
            .prepare_action(action, &scope)
            .and_then(|original| original.advance(&scope, None))
        {
            Ok(worth_ui::facade::query_binding::WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::Committed(execution),
            )) => execution,
            Ok(worth_ui::facade::query_binding::WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::DeniedRevisionMismatch { .. },
            )) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                return Ok(PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::CurrentRevisionLost,
                ));
            }
            Ok(_) => unreachable!("original action construction retains its family"),
            Err(detail) => {
                self.state =
                    PlatformPulseQueryOwnerState::Mutation(PlatformPulseQueryMutation::new(
                        detail,
                        PlatformPulseQueryMutationPurpose::CurrentAction,
                    ));
                return Ok(PlatformPulseQueryActionOutcome::Pending);
            }
        };
        let (publication, evidence) = execution.into_parts();
        let observation = publication
            .into_projection_observation()
            .map_err(PlatformPulseQueryLifecycleDenial::Projection)?;
        self.retain_awaiting(&observation);
        Ok(PlatformPulseQueryActionOutcome::Executed {
            evidence,
            observation,
        })
    }

    pub(crate) fn execute_denied_action(
        &mut self,
        status: impl Into<String>,
        idempotency_session: u64,
        idempotency_lineage: u64,
    ) -> Result<PlatformPulseQueryActionOutcome, PlatformPulseQueryLifecycleDenial> {
        self.require_live()?;
        let submitted_revision = self.current_source_revision()?.checked_add(1).unwrap_or(0);
        let action = WorthUiStatusActionRequest::new(
            submitted_revision,
            status,
            idempotency_session,
            idempotency_lineage,
        )
        .map_err(PlatformPulseQueryLifecycleDenial::ActionRequest)?;
        let scope = self.next_request_scope();
        match self
            .owner
            .prepare_action(action, &scope)
            .and_then(|original| original.advance(&scope, None))
        {
            Ok(worth_ui::facade::query_binding::WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::DeniedRevisionMismatch {
                    active_revision,
                    submitted_revision,
                    ..
                },
            )) => Ok(PlatformPulseQueryActionOutcome::Denied {
                denial: WorthUiScalarProjectionActionPreconditionDenial::SourceRevisionMismatch,
                active_query_source_revision: active_revision,
                submitted_query_source_revision: submitted_revision,
            }),
            Ok(worth_ui::facade::query_binding::WorthUiStatusMutationResult::Action(
                WorthUiStatusActionOutcome::Committed(_),
            )) => {
                self.state = PlatformPulseQueryOwnerState::Indeterminate;
                Ok(PlatformPulseQueryActionOutcome::Indeterminate(
                    PlatformPulseQueryActionIndeterminate::DeniedActionCommitted,
                ))
            }
            Ok(_) => unreachable!("original action construction retains its family"),
            Err(detail) => {
                self.state =
                    PlatformPulseQueryOwnerState::Mutation(PlatformPulseQueryMutation::new(
                        detail,
                        PlatformPulseQueryMutationPurpose::DeniedAction,
                    ));
                Ok(PlatformPulseQueryActionOutcome::Pending)
            }
        }
    }

    fn current_source_revision(&self) -> Result<u64, PlatformPulseQueryLifecycleDenial> {
        self.require_live()?;
        self.owner
            .read_status()
            .map(|publication| publication.value().revision)
            .map_err(PlatformPulseQueryLifecycleDenial::Owner)
    }
}
